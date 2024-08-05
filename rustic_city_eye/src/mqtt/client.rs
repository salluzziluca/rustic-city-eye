use rand::{thread_rng, Rng};
use std::{
    net::{Shutdown, TcpStream},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use crate::{
    mqtt::{
        broker_message::BrokerMessage, client_message::ClientMessage,
        messages_config::MessagesConfig, protocol_error::ProtocolError,
    },
    utils::threadpool::ThreadPool,
};

use super::{client_message, client_return::ClientReturn, stream_operation::StreamOperation};

pub trait ClientTrait {
    fn client_run(&mut self) -> Result<(), ProtocolError>;
    fn assign_packet_id(&self) -> u16;
    fn get_publish_end_channel(
        &self,
    ) -> Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Box<(dyn MessagesConfig + Send + 'static)>>>>;
    fn get_client_id(&self) -> String;
    fn disconnect_client(&self) -> Result<(), ProtocolError>;
}

#[derive(Debug, Clone)]
pub struct Client {
    receiver_channel: Arc<Mutex<Receiver<Box<dyn MessagesConfig + Send>>>>,

    // stream es el socket que se conecta al broker
    stream: Arc<TcpStream>,

    // las subscriptions es un vector de topics a los que el cliente está subscrito
    pub subscriptions: Arc<Mutex<Vec<String>>>,

    // client_id es el identificador del cliente
    pub client_id: String,

    pub packets_ids: Arc<Mutex<Vec<u16>>>,

    sender_channel: Option<Sender<ClientMessage>>,
}

impl Client {
    /// Se intenta conectar al servidor corriendo en address.
    ///
    /// Si la conexion es exitosa, inmediatamente envia un packet del tipo Connect(que
    /// ingresa como parametro).
    ///
    /// Si el enviado del Connect es exitoso, se espera una respuesta del Broker, la cual
    /// debe ser un Connack packet.
    ///
    /// Al recibir un Connack, se ve su reason_code, y si este es 0x00(conexion exitosa), se crea la instancia
    /// del Client.
    pub fn new(
        receiver_channel: Receiver<Box<dyn MessagesConfig + Send>>,
        address: String,
        connect: client_message::Connect,
        sender_channel: Sender<ClientMessage>,
    ) -> Result<Client, ProtocolError> {
        let mut stream = match TcpStream::connect(address) {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::ConectionError),
        };

        let connect_message = ClientMessage::Connect(connect.clone());

        println!("Enviando connect message to broker");

        match connect_message.write_to(&mut stream) {
            Ok(()) => println!("Connect message enviado"),
            Err(_) => println!("Error al enviar connect message"),
        }

        if let Ok(message) = BrokerMessage::read_from(&mut stream) {
            match message {
                BrokerMessage::Connack {
                    session_present: _,
                    reason_code,
                    properties: _,
                } => {
                    println!("Recibí un Connack");
                    match reason_code {
                        0x00_u8 => {
                            println!("Conexion exitosa!");

                            let client_id = connect.client_id;
                            Ok(Client {
                                receiver_channel: Arc::new(Mutex::new(receiver_channel)),
                                stream: Arc::new(stream),
                                subscriptions: Arc::new(Mutex::new(Vec::new())),
                                packets_ids: Arc::new(Mutex::new(Vec::new())),
                                sender_channel: Some(sender_channel),
                                client_id,
                            })
                        }
                        _ => {
                            println!("Connack con reason code {}", reason_code);
                            Err(ProtocolError::AuthError)
                        }
                    }
                }
                _ => Err(ProtocolError::ExpectedConnack),
            }
        } else {
            Err(ProtocolError::NotReceivedMessageError)
        }
    }

    pub fn run_client(&mut self) -> Result<(), ProtocolError> {
        let threadpool = ThreadPool::new(10);

        let stream_write_half = Arc::clone(&self.stream);
        let stream_read_half = Arc::clone(&self.stream);

        let client_id_clone = self.client_id.clone();

        let (sender, receiver) = mpsc::channel();
        let sender_clone = sender.clone();
        let (disconnect_from_broker_sender, disconnect_from_broker_receiver) = mpsc::channel();
        let (disconnect_from_client_sender, disconnect_from_client_receiver) = mpsc::channel();
        let (disconnect_from_client_sender_two, disconnect_from_client_receiver_two) =
            mpsc::channel();
        let (puback_notify_sender, puback_notify_receiver) = mpsc::channel();

        let receiver_channel_ref = Arc::clone(&self.receiver_channel);
        let subscriptions_ref = Arc::clone(&self.subscriptions);
        let (pending_id_messages_sender, pending_id_messages_receiver) = mpsc::channel();
        let (message_from_stream_sender, message_from_stream_receiver) = mpsc::channel();
        let sender_to_client_channel_clone = self.sender_channel.clone();

        let _handle_stream_operations = threadpool.execute(move || {
            let _ = Client::handle_stream_operations(stream_write_half, receiver);
        });

        let _handle_stream_readings = threadpool.execute(move || {
            let _ = Client::handle_stream_readings(
                stream_read_half,
                message_from_stream_sender,
                puback_notify_sender,
                disconnect_from_client_receiver,
            );
        });

        let _handle_incoming_messages = threadpool.execute(move || {
            let _ = Client::handle_incoming_messages(
                sender,
                receiver_channel_ref,
                pending_id_messages_sender,
                disconnect_from_broker_receiver,
                subscriptions_ref,
                puback_notify_receiver,
                disconnect_from_client_sender,
                disconnect_from_client_sender_two,
            );
        });

        if let Some(sender_to_client) = sender_to_client_channel_clone {
            let _handle_reading_messages = threadpool.execute(move || {
                let _ = Client::handle_reading_messages(
                    message_from_stream_receiver,
                    sender_clone,
                    sender_to_client,
                    pending_id_messages_receiver,
                    disconnect_from_client_receiver_two,
                    disconnect_from_broker_sender,
                    client_id_clone,
                );
            });
        }
        Ok(())
    }

    fn handle_stream_operations(
        stream: Arc<TcpStream>,
        receiver: Receiver<StreamOperation>,
    ) -> Result<(), ProtocolError> {
        loop {
            if let Ok(command) = receiver.recv() {
                match command {
                    StreamOperation::WriteClientMessage(message) => {
                        match message.write_to(&*stream) {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("{}", e);
                                return Err(e);
                            }
                        };
                    }
                    StreamOperation::WriteAndDisconnect(message) => {
                        match message.write_to(&*stream) {
                            Ok(_) => {
                                match stream.shutdown(Shutdown::Both) {
                                    Ok(_) => break,
                                    Err(e) => {
                                        return Err(ProtocolError::ShutdownError(e.to_string()));
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("{}", e);
                                return Err(e);
                            }
                        };
                    }
                    StreamOperation::ShutdownStream => match stream.shutdown(Shutdown::Both) {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            return Err(ProtocolError::ShutdownError(e.to_string()));
                        }
                    },
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub fn handle_stream_readings(
        stream: Arc<TcpStream>,
        message_from_stream_sender: Sender<BrokerMessage>,
        puback_notify_sender: Sender<bool>,
        disconnect_from_client_receiver: Receiver<()>,
    ) -> Result<(), ProtocolError> {
        loop {
            if disconnect_from_client_receiver.try_recv().is_ok() {
                break;
            }

            if let Ok(message) = BrokerMessage::read_from(&*stream) {
                match message {
                    BrokerMessage::Disconnect {
                        reason_code,
                        session_expiry_interval,
                        reason_string,
                        user_properties,
                    } => {
                        let disconnect = BrokerMessage::Disconnect {
                            reason_code,
                            session_expiry_interval,
                            reason_string,
                            user_properties,
                        };
                        message_from_stream_sender.send(disconnect).unwrap();
                        break;
                    }
                    BrokerMessage::Puback {
                        packet_id_msb: _,
                        packet_id_lsb: _,
                        reason_code: _,
                    } => {
                        message_from_stream_sender.send(message).unwrap();
                        puback_notify_sender.send(true).unwrap();
                    }
                    _ => message_from_stream_sender.send(message).unwrap(),
                }
            }
        }
        Ok(())
    }

    /// Toma las configuraciones de los packets que quiere enviar el usuario al sistema a traves de un canal.
    ///
    /// Una vez con la configuracion del packet, se convierte en el packet que se enviara al sistema, y se a traves de un canal
    /// se envia a otro thread encargado de escribirlo en el Stream del Client.
    ///
    /// En caso de recibir la configuracion de un packet Disconnect, se envia el packet al thread igualmente, pero el loop de esta funcion
    /// termina y se retorna un Ok en caso de no haber errores.
    fn handle_incoming_messages(
        sender: Sender<StreamOperation>,
        receiver_channel: Arc<Mutex<Receiver<Box<dyn MessagesConfig + Send>>>>,
        pending_id_messages_sender: Sender<u16>,
        disconnect_from_broker_receiver: Receiver<()>,
        subscriptions_ref: Arc<Mutex<Vec<String>>>,
        puback_notify_receiver: Receiver<bool>,
        disconnect_from_client_sender: Sender<()>,
        disconnect_from_client_sender_two: Sender<()>,
    ) -> Result<(), ProtocolError> {
        loop {
            if disconnect_from_broker_receiver.try_recv().is_ok() {
                break;
            }

            let lock = match receiver_channel.lock() {
                Ok(lock) => lock,
                Err(_) => break,
            };

            if let Ok(message_config) = lock.try_recv() {
                let mut rng = thread_rng();
                let packet_id = rng.gen::<u16>();

                let message = message_config.parse_message(packet_id);

                match message {
                    ClientMessage::Publish {
                        packet_id,
                        topic_name,
                        qos,
                        retain_flag,
                        payload,
                        dup_flag,
                        properties,
                    } => {
                        let publish = ClientMessage::Publish {
                            packet_id,
                            topic_name: topic_name.clone(),
                            qos,
                            retain_flag,
                            payload: payload.clone(),
                            dup_flag,
                            properties: properties.clone(),
                        };
                        match sender.send(StreamOperation::WriteClientMessage(publish.clone())) {
                            Ok(_) => match pending_id_messages_sender.send(packet_id) {
                                Ok(_) => {
                                    if qos == 1 {
                                        loop {
                                            thread::sleep(Duration::from_millis(20));

                                            if let Ok(puback) = puback_notify_receiver.try_recv() {
                                                if !puback {
                                                    let publish = ClientMessage::Publish {
                                                        packet_id,
                                                        topic_name: topic_name.clone(),
                                                        qos,
                                                        retain_flag,
                                                        payload: payload.clone(),
                                                        dup_flag: dup_flag + 1,
                                                        properties: properties.clone(),
                                                    };

                                                    match sender.send(
                                                        StreamOperation::WriteClientMessage(
                                                            publish,
                                                        ),
                                                    ) {
                                                        Ok(_) => match pending_id_messages_sender
                                                            .send(packet_id)
                                                        {
                                                            Ok(_) => {}
                                                            Err(e) => {
                                                                return Err(
                                                                    ProtocolError::SendError(
                                                                        e.to_string(),
                                                                    ),
                                                                );
                                                            }
                                                        },
                                                        Err(e) => {
                                                            return Err(ProtocolError::SendError(
                                                                e.to_string(),
                                                            ));
                                                        }
                                                    }
                                                } else {
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    return Err(ProtocolError::SendError(e.to_string()));
                                }
                            },
                            Err(e) => {
                                return Err(ProtocolError::SendError(e.to_string()));
                            }
                        }
                    }
                    ClientMessage::Subscribe {
                        packet_id: _,
                        properties: _,
                        ref payload,
                    } => {
                        let topic_new = payload.topic.to_string();

                        if let Ok(mut guard) = subscriptions_ref.lock() {
                            guard.push(topic_new);
                        };

                        match sender.send(StreamOperation::WriteClientMessage(message)) {
                            Ok(_) => match pending_id_messages_sender.send(packet_id) {
                                Ok(_) => {}
                                Err(e) => {
                                    return Err(ProtocolError::SendError(e.to_string()));
                                }
                            },
                            Err(e) => {
                                return Err(ProtocolError::SendError(e.to_string()));
                            }
                        }
                    }
                    ClientMessage::Unsubscribe {
                        packet_id: _,
                        properties: _,
                        ref payload,
                    } => {
                        let topic_new = payload.topic.to_string();
                        if let Ok(mut guard) = subscriptions_ref.lock() {
                            guard.retain(|x| x != &topic_new);
                        };

                        match sender.send(StreamOperation::WriteClientMessage(message)) {
                            Ok(_) => match pending_id_messages_sender.send(packet_id) {
                                Ok(_) => {}
                                Err(e) => {
                                    return Err(ProtocolError::SendError(e.to_string()));
                                }
                            },
                            Err(e) => {
                                return Err(ProtocolError::SendError(e.to_string()));
                            }
                        }
                    }
                    ClientMessage::Disconnect {
                        reason_code: _,
                        session_expiry_interval,
                        reason_string,
                        client_id,
                    } => {
                        let disconnect = Client::handle_disconnect(
                            client_id,
                            reason_string,
                            session_expiry_interval,
                        );

                        disconnect_from_client_sender.send(()).unwrap();
                        disconnect_from_client_sender_two.send(()).unwrap();

                        match sender.send(StreamOperation::WriteAndDisconnect(disconnect)) {
                            Ok(_) => {
                                break;
                            }
                            Err(e) => {
                                return Err(ProtocolError::SendError(e.to_string()));
                            }
                        }
                    }
                    _ => match sender.send(StreamOperation::WriteClientMessage(message)) {
                        Ok(_) => match pending_id_messages_sender.send(packet_id) {
                            Ok(_) => {}
                            Err(e) => {
                                return Err(ProtocolError::SendError(e.to_string()));
                            }
                        },
                        Err(e) => {
                            return Err(ProtocolError::SendError(e.to_string()));
                        }
                    },
                }
            }
        }
        Ok(())
    }

    fn handle_reading_messages(
        message_from_stream_receiver: Receiver<BrokerMessage>,
        sender: Sender<StreamOperation>,
        sender_to_client: Sender<ClientMessage>,
        pending_id_messages_receiver: Receiver<u16>,
        disconnect_receiver: Receiver<()>,
        disconnect_from_broker_sender: Sender<()>,
        client_id: String,
    ) -> Result<(), ProtocolError> {
        let mut pending_messages: Vec<u16> = Vec::new();

        loop {
            if disconnect_receiver.try_recv().is_ok() {
                break;
            }

            if let Ok(packet) = pending_id_messages_receiver.try_recv() {
                if !pending_messages.contains(&packet) {
                    pending_messages.push(packet);
                }
            }

            if let Ok(message) = message_from_stream_receiver.try_recv() {
                match Client::handle_message(
                    message,
                    &mut pending_messages,
                    sender.clone(),
                    sender_to_client.clone(),
                    disconnect_from_broker_sender.clone(),
                    client_id.clone(),
                ) {
                    Ok(ClientReturn::DisconnectRecieved) => {
                        println!("Disconnect received");
                        break;
                    }
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }

    /// Se encarga de procesar un mensaje proveniente del Broker, y realiza las operaciones correspondientes
    /// con los canales, a traves de los cuales se comunica con los demas hilos del Client, y con el mismo
    /// usuario de la API.
    pub fn handle_message(
        message: BrokerMessage,
        pending_messages: &mut Vec<u16>,
        sender: Sender<StreamOperation>,
        sender_to_client: Sender<ClientMessage>,
        disconnect_from_broker_sender: Sender<()>,
        client_id: String,
    ) -> Result<ClientReturn, ProtocolError> {
        match message {
            BrokerMessage::PublishDelivery {
                packet_id,
                topic_name,
                qos,
                retain_flag,
                dup_flag,
                properties,
                payload,
            } => {
                match sender_to_client.send(ClientMessage::Publish {
                    packet_id,
                    topic_name,
                    qos,
                    retain_flag,
                    dup_flag,
                    properties,
                    payload,
                }) {
                    Ok(_) => Ok(ClientReturn::PublishDeliveryRecieved),
                    Err(e) => {
                        println!("Error al enviar publish al sistema: {:?}", e);
                        Err(ProtocolError::PublishError)
                    }
                }
            }
            BrokerMessage::Puback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
            } => {
                Client::process_packet_id_response(pending_messages, packet_id_msb, packet_id_lsb);
                Ok(ClientReturn::PubackRecieved)
            }
            BrokerMessage::Suback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
            } => {
                Client::process_packet_id_response(pending_messages, packet_id_msb, packet_id_lsb);
                Ok(ClientReturn::SubackRecieved)
            }
            BrokerMessage::Unsuback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
            } => {
                Client::process_packet_id_response(pending_messages, packet_id_msb, packet_id_lsb);
                Ok(ClientReturn::UnsubackRecieved)
            }
            BrokerMessage::Disconnect {
                reason_code,
                session_expiry_interval,
                reason_string,
                user_properties: _,
            } => {
                match sender_to_client.send(ClientMessage::Disconnect {
                    reason_code,
                    session_expiry_interval,
                    reason_string,
                    client_id: client_id.clone(),
                }) {
                    Ok(_) => {
                        sender.send(StreamOperation::ShutdownStream).unwrap();
                        disconnect_from_broker_sender.send(()).unwrap();
                        Ok(ClientReturn::DisconnectRecieved)
                    }
                    Err(e) => {
                        println!("Error al enviar disconnect al sistema: {:?}", e);
                        Err(ProtocolError::DisconnectError)
                    }
                }
            }
            BrokerMessage::Connack {
                session_present: _,
                reason_code: _,
                properties: _,
            } => Ok(ClientReturn::ConnackReceived),

            BrokerMessage::Auth {
                reason_code: _,
                authentication_method: _,
                authentication_data: _,
                reason_string: _,
                user_properties: _,
            } => Ok(ClientReturn::AuthRecieved),

            BrokerMessage::Pingresp => Ok(ClientReturn::PingrespRecieved),
        }
    }

    /// Recibe un string que indica la razón de la desconexión y construye el Disconnect packet correspondiente
    /// segun el str reason recibido.
    fn handle_disconnect(
        client_id: String,
        reason: String,
        session_expiry_interval: u32,
    ) -> ClientMessage {
        let reason_code: u8;
        let reason_string: String;
        match reason.as_str() {
            "normal" => {
                reason_code = 0x00;
                reason_string =
                    "Close the connection normally. Do not send the Will Message.".to_string();
            }
            "with_will" => {
                reason_code = 0x04;
                reason_string = "The Client wishes to disconnect but requires that the Server also publishes its Will Message"
                    .to_string();
            }
            _ => {
                reason_code = 0x80;
                reason_string = "The Connection is closed but the sender either does not wish to reveal reason or none of the other Reason Codes apply. "
                    .to_string();
            }
        }
        ClientMessage::Disconnect {
            reason_code,
            session_expiry_interval,
            reason_string,
            client_id: client_id.to_string(),
        }
    }

    fn process_packet_id_response(
        pending_messages: &mut Vec<u16>,
        packet_id_msb: u8,
        packet_id_lsb: u8,
    ) {
        pending_messages.retain(|&pending_message| {
            let packet_id_bytes: [u8; 2] = pending_message.to_be_bytes();
            !(packet_id_bytes[0] == packet_id_msb && packet_id_bytes[1] == packet_id_lsb)
        });
    }

    pub fn get_publish_end_channel(
        &self,
    ) -> Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Box<(dyn MessagesConfig + Send + 'static)>>>>
    {
        self.receiver_channel.clone()
    }

    ///Asigna un id random
    pub fn assign_packet_id(&self) -> u16 {
        let mut rng = rand::thread_rng();
        let mut packet_ids = match self.packets_ids.lock() {
            Ok(packet_ids) => packet_ids,
            Err(_) => return 0,
        };

        let mut packet_id: u16;
        loop {
            packet_id = rng.gen();
            if packet_id != 0 && !packet_ids.contains(&packet_id) {
                packet_ids.push(packet_id);
                break;
            }
        }
        packet_id
    }
}

impl ClientTrait for Client {
    fn client_run(&mut self) -> Result<(), ProtocolError> {
        self.run_client()
    }

    fn assign_packet_id(&self) -> u16 {
        self.assign_packet_id()
    }

    fn get_publish_end_channel(
        &self,
    ) -> Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Box<(dyn MessagesConfig + Send + 'static)>>>>
    {
        self.get_publish_end_channel()
    }
    fn get_client_id(&self) -> String {
        self.client_id.clone()
    }

    fn disconnect_client(&self) -> Result<(), ProtocolError> {
        // let lock = match self.stream.lock() {
        //     Ok(lock) => lock,
        //     Err(_) => return Err(ProtocolError::StreamError),
        // };

        //let writer = &mut lock;

        // writer.flush().map_err(|_| ProtocolError::WriteError)?;

        // match self.stream.shutdown(Shutdown::Both) {
        //     Ok(_) => Ok(()),
        //     Err(e) => {
        //         println!("Client: Error while shutting down stream: {:?}", e);
        //         Err(ProtocolError::ShutdownError(e.to_string()))
        //     }
        // }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::{sync::Condvar, thread};

    use mpsc::channel;

    use crate::{
        monitoring::incident::Incident,
        mqtt::{
            broker::Broker,
            connack_properties::ConnackProperties,
            disconnect_config::DisconnectConfig,
            publish::{
                publish_config::PublishConfig,
                publish_properties::{PublishProperties, TopicProperties},
            },
        },
        utils::{
            incident_payload::IncidentPayload, location::Location, payload_types::PayloadTypes,
        },
    };

    use super::*;

    #[test]
    fn test_assign_packet_id() {
        let args = vec!["127.0.0.1".to_string(), "5047".to_string()];
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => panic!("Error creating broker: {:?}", e),
        };

        let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
        let server_ready_clone = server_ready.clone();
        thread::spawn(move || {
            {
                let (lock, cvar) = &*server_ready_clone;
                let mut ready = lock.lock().unwrap();
                *ready = true;
                cvar.notify_all();
            }
            broker.server_run().unwrap();
        });

        // Wait for the server to start
        {
            let (lock, cvar) = &*server_ready;
            let mut ready = lock.lock().unwrap();
            while !*ready {
                ready = cvar.wait(ready).unwrap();
            }
        }
        let handle = thread::spawn(move || {
            let connect_config =
                client_message::Connect::read_connect_config("src/drones/connect_config.json")
                    .unwrap();
            let (_, rx) = mpsc::channel();
            let (tx2, _) = mpsc::channel();
            let address = "127.0.0.1:5047";
            let client = Client::new(rx, address.to_string(), connect_config, tx2).unwrap();
            let packet_id = client.assign_packet_id();
            assert_ne!(packet_id, 0);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_si_el_id_de_paquete_es_unico() {
        let args = vec!["127.0.0.1".to_string(), "5039".to_string()];
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => panic!("Error creating broker: {:?}", e),
        };

        let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
        let server_ready_clone = server_ready.clone();
        thread::spawn(move || {
            {
                let (lock, cvar) = &*server_ready_clone;
                let mut ready = lock.lock().unwrap();
                *ready = true;
                cvar.notify_all();
            }
            let _ = broker.server_run();
        });

        // Wait for the server to start
        {
            let (lock, cvar) = &*server_ready;
            let mut ready = lock.lock().unwrap();
            while !*ready {
                ready = cvar.wait(ready).unwrap();
            }
        }
        let handle = thread::spawn(move || {
            let connect_config =
                client_message::Connect::read_connect_config("src/drones/connect_config.json")
                    .unwrap();
            let (_, rx) = mpsc::channel();
            let (tx2, _) = mpsc::channel();
            let address = "127.0.0.1:5039";
            let client = Client::new(rx, address.to_string(), connect_config, tx2).unwrap();
            let packet_id = client.assign_packet_id();
            let packet_id_2 = client.assign_packet_id();
            assert_ne!(packet_id, packet_id_2);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_si_el_id_de_paquete_es_distinto_de_cero() {
        let args = vec!["127.0.0.1".to_string(), "8080".to_string()];
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => panic!("Error creating broker: {:?}", e),
        };

        let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
        let server_ready_clone = server_ready.clone();
        thread::spawn(move || {
            {
                let (lock, cvar) = &*server_ready_clone;
                let mut ready = lock.lock().unwrap();
                *ready = true;
                cvar.notify_all();
            }
            let _ = broker.server_run();
        });

        // Wait for the server to start
        {
            let (lock, cvar) = &*server_ready;
            let mut ready = lock.lock().unwrap();
            while !*ready {
                ready = cvar.wait(ready).unwrap();
            }
        }
        let handle = thread::spawn(move || {
            let connect_config =
                client_message::Connect::read_connect_config("src/drones/connect_config.json")
                    .unwrap();
            let (_, rx) = mpsc::channel();
            let (tx2, _) = mpsc::channel();
            let address = "127.0.0.1:8080";
            let client = Client::new(rx, address.to_string(), connect_config, tx2).unwrap();
            let packet_id = client.assign_packet_id();
            assert_ne!(packet_id, 0);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_04_recibir_connack() {
        let properties = ConnackProperties {
            session_expiry_interval: 0,
            receive_maximum: 0,
            maximum_packet_size: 0,
            topic_alias_maximum: 0,
            user_properties: vec![],
            authentication_method: "password-based".to_string(),
            authentication_data: vec![],
            assigned_client_identifier: "none".to_string(),
            maximum_qos: true,
            reason_string: "buendia".to_string(),
            wildcard_subscription_available: false,
            subscription_identifier_available: false,
            shared_subscription_available: false,
            server_keep_alive: 0,
            response_information: "none".to_string(),
            server_reference: "none".to_string(),
            retain_available: false,
        };

        let connack = BrokerMessage::Connack {
            session_present: true,
            reason_code: 0x00_u8,
            properties,
        };

        let mut pending_messages: Vec<u16> = Vec::new();
        let (sender, _) = mpsc::channel();
        let (tx, _) = mpsc::channel();
        let (disconnect_from_broker_sender, _) = mpsc::channel();
        let result = Client::handle_message(
            connack,
            &mut pending_messages,
            sender,
            tx,
            disconnect_from_broker_sender,
            "monitoreo".to_string(),
        );

        assert_eq!(result.unwrap(), ClientReturn::ConnackReceived);
    }

    #[test]
    fn test_05_recibir_puback() {
        let puback = BrokerMessage::Puback {
            packet_id_msb: 1,
            packet_id_lsb: 5,
            reason_code: 1,
        };

        let mut pending_messages: Vec<u16> = Vec::new();
        let (sender, _) = channel();
        let (tx, _) = channel();
        let (disconnect_from_broker_sender, _) = mpsc::channel();

        let result = Client::handle_message(
            puback,
            &mut pending_messages,
            sender,
            tx,
            disconnect_from_broker_sender,
            "juancito".to_string(),
        );

        assert_eq!(result.unwrap(), ClientReturn::PubackRecieved);
    }

    #[test]
    fn test_06_recibo_publish_delivery() {
        let mut pending_messages: Vec<u16> = Vec::new();
        let (sender, _) = channel();
        let (tx, rx) = channel();
        let (disconnect_from_broker_sender, _) = mpsc::channel();

        let topic = TopicProperties {
            topic_alias: 1,
            response_topic: "topic".to_string(),
        };

        let publish_propreties = PublishProperties {
            payload_format_indicator: 1,
            message_expiry_interval: 2,
            topic_properties: topic,
            correlation_data: vec![1, 2, 3],
            user_property: "propiedad".to_string(),
            subscription_identifier: 3,
            content_type: "content".to_string(),
        };
        let location = Location::new(1.1, 1.12);
        let new = Incident::new(location);
        let incident_payload = IncidentPayload::new(new);
        let pub_delivery = BrokerMessage::PublishDelivery {
            packet_id: 1,
            topic_name: "topic".to_string(),
            qos: 1,
            retain_flag: 2,
            payload: PayloadTypes::IncidentLocation(incident_payload),
            dup_flag: 4,
            properties: publish_propreties,
        };

        thread::spawn(move || loop {
            if let Ok(_) = rx.try_recv() {
                println!("Recibi un delivery");
            }
        });

        let result = Client::handle_message(
            pub_delivery,
            &mut pending_messages,
            sender,
            tx,
            disconnect_from_broker_sender,
            "juancito".to_string(),
        );

        assert_eq!(result.unwrap(), ClientReturn::PublishDeliveryRecieved);
    }

    #[test]
    fn test_07_recibo_disconnect() {
        let mut pending_messages: Vec<u16> = Vec::new();
        let (sender, rx) = channel();
        let (tx, rx2) = channel();
        let (disconnect_from_broker_sender, rx3) = mpsc::channel();

        let disconnect = BrokerMessage::Disconnect {
            reason_code: 1,
            session_expiry_interval: 1,
            reason_string: "pasaron_cosas".to_string(),
            user_properties: vec![("propiedad".to_string(), "valor".to_string())],
        };

        thread::spawn(move || loop {
            if rx.try_recv().is_ok() && rx2.try_recv().is_ok() && rx3.try_recv().is_ok() {
                println!("Recibi un delivery");
            }
        });

        let result = Client::handle_message(
            disconnect,
            &mut pending_messages,
            sender,
            tx,
            disconnect_from_broker_sender,
            "juancito".to_string(),
        );

        assert_eq!(result.unwrap(), ClientReturn::DisconnectRecieved);
    }

    #[test]
    fn test_08_recibir_resto_de_packetes_ok() {
        let mut pending_messages: Vec<u16> = Vec::new();
        let (sender, _) = channel();
        let (tx, _) = channel();
        let (disconnect_from_broker_sender, _) = mpsc::channel();

        let suback = BrokerMessage::Suback {
            packet_id_msb: 3,
            packet_id_lsb: 1,
            reason_code: 3,
        };

        let result = Client::handle_message(
            suback,
            &mut pending_messages,
            sender.clone(),
            tx.clone(),
            disconnect_from_broker_sender.clone(),
            "juancito".to_string(),
        );

        assert_eq!(result.unwrap(), ClientReturn::SubackRecieved);

        let unsuback = BrokerMessage::Unsuback {
            packet_id_msb: 1,
            packet_id_lsb: 1,
            reason_code: 1,
        };

        let result = Client::handle_message(
            unsuback,
            &mut pending_messages,
            sender.clone(),
            tx.clone(),
            disconnect_from_broker_sender.clone(),
            "juancito".to_string(),
        );
        assert_eq!(result.unwrap(), ClientReturn::UnsubackRecieved);

        let pingresp = BrokerMessage::Pingresp;

        let result = Client::handle_message(
            pingresp,
            &mut pending_messages,
            sender.clone(),
            tx.clone(),
            disconnect_from_broker_sender.clone(),
            "juancito".to_string(),
        );

        assert_eq!(result.unwrap(), ClientReturn::PingrespRecieved);

        let auth = BrokerMessage::Auth {
            reason_code: 0x00_u8,
            authentication_method: "password-based".to_string(),
            authentication_data: vec![0x00_u8, 0x01_u8],
            reason_string: "success".to_string(),
            user_properties: vec![("juan".to_string(), "hola".to_string())],
        };

        let result = Client::handle_message(
            auth,
            &mut pending_messages,
            sender,
            tx,
            disconnect_from_broker_sender,
            "juancito".to_string(),
        );

        assert_eq!(result.unwrap(), ClientReturn::AuthRecieved);
    }

    #[test]
    fn test_09_leo_mensajes_hasta_que_llega_un_disconnect_ok() {
        let (message_sender, message_receiver) = channel();
        let (stream_sender, _stream_receiver) = channel();
        let (client_sender, _client_receiver) = channel();
        let (_pending_id_sender, pending_id_receiver) = channel();
        let (_disconnect_sender, disconnect_receiver) = channel();
        let (disconnect_from_broker_sender, _disconnect_from_broker_receiver) = channel();

        let client_id = "test_client".to_string();

        let handle = thread::spawn(move || {
            Client::handle_reading_messages(
                message_receiver,
                stream_sender,
                client_sender,
                pending_id_receiver,
                disconnect_receiver,
                disconnect_from_broker_sender,
                client_id,
            )
        });

        let suback = BrokerMessage::Suback {
            packet_id_msb: 3,
            packet_id_lsb: 1,
            reason_code: 3,
        };
        let unsuback = BrokerMessage::Unsuback {
            packet_id_msb: 1,
            packet_id_lsb: 1,
            reason_code: 1,
        };
        let disconnect = BrokerMessage::Disconnect {
            reason_code: 1,
            session_expiry_interval: 1,
            reason_string: "pasaron_cosas".to_string(),
            user_properties: vec![("propiedad".to_string(), "valor".to_string())],
        };

        message_sender.send(suback).unwrap();
        message_sender.send(unsuback).unwrap();
        message_sender.send(disconnect).unwrap();

        let result = handle.join().unwrap();
        assert!(result.is_ok());
    }

    #[test]
    fn test_10_manejo_mensajes_que_quiere_enviar_el_user_hasta_recibir_disconnect() {
        let (stream_sender, _stream_receiver) = channel();
        let (pending_id_sender, _pending_id_receiver) = channel();
        let (_disconnect_sender, disconnect_receiver) = channel();
        let (_puback_notify_sender, puback_notify_receiver) = channel();

        let subscriptions = Arc::new(Mutex::new(Vec::new()));
        let (receiver_channel_sender, receiver_channel_receiver) = channel();
        let receiver_channel = Arc::new(Mutex::new(receiver_channel_receiver));

        let incident = Incident::new(Location {
            long: 1.2,
            lat: 2.1,
        });
        let incident_payload = IncidentPayload::new(incident);
        let publish_config = PublishConfig::read_config(
            "src/monitoring/publish_incident_config.json",
            PayloadTypes::IncidentLocation(incident_payload),
        )
        .unwrap();
        let disconnect_config =
            DisconnectConfig::new(0x00_u8, 1, "normal".to_string(), "test_client".to_string());

        let handle = thread::spawn({
            let receiver_channel = Arc::clone(&receiver_channel);
            let subscriptions = Arc::clone(&subscriptions);
            let (disconnect_from_client_sender, _) = mpsc::channel();
            let (disconnect_from_client_sender_two, _) = mpsc::channel();

            move || {
                Client::handle_incoming_messages(
                    stream_sender,
                    receiver_channel,
                    pending_id_sender,
                    disconnect_receiver,
                    subscriptions,
                    puback_notify_receiver,
                    disconnect_from_client_sender,
                    disconnect_from_client_sender_two
                )
            }
        });

        receiver_channel_sender
            .send(Box::new(publish_config))
            .unwrap();

        receiver_channel_sender
            .send(Box::new(disconnect_config))
            .unwrap();

        let result = handle.join().unwrap();
        assert!(result.is_ok());
    }
}
