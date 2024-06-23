use rand::Rng;
use std::{
    net::TcpStream,
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

use super::{client_message, client_return::ClientReturn, error::ClientError};

pub trait ClientTrait {
    fn client_run(&mut self) -> Result<(), ProtocolError>;
    fn clone_box(&self) -> Box<dyn ClientTrait>;
    fn assign_packet_id(&self) -> u16;
    fn get_publish_end_channel(
        &self,
    ) -> Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Box<(dyn MessagesConfig + Send + 'static)>>>>;
}
impl Clone for Box<dyn ClientTrait> {
    fn clone(&self) -> Box<dyn ClientTrait> {
        self.clone_box()
    }
}
#[derive(Debug, Clone)]
pub struct Client {
    receiver_channel: Arc<Mutex<Receiver<Box<dyn MessagesConfig + Send>>>>,

    // stream es el socket que se conecta al broker
    stream: Arc<Mutex<TcpStream>>,

    // las subscriptions es un vector de topics a los que el cliente está subscrito
    pub subscriptions: Arc<Mutex<Vec<String>>>,

    // client_id es el identificador del cliente
    pub client_id: String,
    // las subscriptions son un hashmap de topic y sub_id
    // user_id: u32,
    pub packets_ids: Arc<Mutex<Vec<u16>>>,

    sender_channel: Sender<ClientMessage>,
}

impl Client {
    /// Se intenta connectar al servidor corriendo en address.
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
        let stream = match TcpStream::connect(address) {
            Ok(stream) => Arc::new(Mutex::new(stream)),
            Err(_) => return Err(ProtocolError::ConectionError),
        };
        let mut stream_lock = match stream.lock() {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::StreamError),
        };

        println!("Enviando connect message to broker");

        match connect.write_to(&mut *stream_lock) {
            Ok(()) => println!("Connect message enviado"),
            Err(_) => println!("Error al enviar connect message"),
        }

        if let Ok(message) = BrokerMessage::read_from(&mut *stream_lock) {
            match message {
                BrokerMessage::Connack {
                    session_present: _,
                    reason_code,
                    properties: _,
                } => {
                    println!("Recibí un Connack");
                    let stream_clone = Arc::clone(&stream);

                    match reason_code {
                        0x00_u8 => {
                            println!("Conexion exitosa!");

                            let client_id = connect.client_id.clone();
                            Ok(Client {
                                receiver_channel: Arc::new(Mutex::new(receiver_channel)),
                                stream: stream_clone,
                                subscriptions: Arc::new(Mutex::new(Vec::new())),
                                packets_ids: Arc::new(Mutex::new(Vec::new())),
                                sender_channel,
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

    /// Publica un mensaje en un topic determinado.
    pub fn publish_message(
        message: ClientMessage,
        mut stream: TcpStream,
        packet_id: u16,
    ) -> Result<u16, ClientError> {
        //chequeo si el mensaje es de tipo publish
        if let ClientMessage::Publish {
            packet_id: _,
            topic_name: _,
            qos: _,
            retain_flag: _,
            payload: _,
            dup_flag: _,
            properties: _,
        } = message
        {
            match message.write_to(&mut stream) {
                Ok(()) => Ok(packet_id),
                Err(_) => Err(ClientError::new("Error al enviar mensaje")),
            }
        } else {
            Err(ClientError::new("El mensaje no es de tipo publish"))
        }
    }

    pub fn subscribe(
        message: ClientMessage,
        packet_id: u16,
        mut stream: TcpStream,
    ) -> Result<u16, ClientError> {
        match message.write_to(&mut stream) {
            Ok(()) => Ok(packet_id),
            Err(_) => Err(ClientError::new("Error al enviar mensaje")),
        }
    }

    pub fn unsubscribe(
        message: ClientMessage,
        mut stream: TcpStream,
        packet_id: u16,
    ) -> Result<u16, ClientError> {
        match message.write_to(&mut stream) {
            Ok(()) => Ok(packet_id),
            Err(_) => Err(ClientError::new("Error al enviar mensaje")),
        }
    }

    fn _assign_subscription_id() -> u8 {
        let mut rng = rand::thread_rng();

        let sub_id: u8 = rng.gen();

        sub_id
    }

    ///recibe un string que indica la razón de la desconexión y un stream y envia un disconnect message al broker
    /// segun el str reason recibido, modifica el reason_code y el reason_string del mensaje
    ///
    /// devuelve el packet_id del mensaje enviado o un ClientError en caso de error
    pub fn handle_disconnect(
        client_id: String,
        reason: &str,
        mut stream: TcpStream,
    ) -> Result<u16, ClientError> {
        let packet_id = 1;
        let reason_code: u8;
        let reason_string: String;
        match reason {
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
        let disconnect = ClientMessage::Disconnect {
            reason_code,
            session_expiry_interval: 0,
            reason_string,
            client_id,
        };

        match disconnect.write_to(&mut stream) {
            Ok(()) => Ok(packet_id),
            Err(_) => Err(ClientError::new("Error al enviar mensaje")),
        }
    }

    /// Se encarga de que el cliente este funcionando correctamente.
    /// El Client debe encargarse de dos tareas: leer mensajes que le lleguen del Broker.
    /// Estos mensajes pueden ser tanto acks (Connack, Puback, etc.)
    /// como pubs que vengan por parte otros clientes (mediante el broker)
    /// a cuyos topics esté subscrito.
    ///
    /// Su segunda tarea es enviar mensajes:
    /// puede enviar mensajes como Publish, Suscribe, etc.
    ///
    /// Las dos tareas del Client se deben ejecutar concurrentemente,
    /// por eso su tilizan dos threads(uno de lectura y otro de escritura),
    /// ambos threads deben compaten el recurso TcpStream.
    ///
    /// El thread de escritura (write_messages) recibe por consola los mensajes a enviar.
    /// Si logra enviar los mensajes correctamente, envia el pacjet id mediante el channel
    ///
    /// El thread de lectura (read_messages) se encarga de leer los mensajes que le llegan del broker.
    pub fn client_run(&mut self) -> Result<(), ProtocolError> {
        let (write_sender, recieve_receiver) = mpsc::channel();
        let (reciever_sender, write_receiver) = mpsc::channel();

        let receiver_channel = self.receiver_channel.clone();

        let desconectar = false;
        let stream_lock = match self.stream.lock() {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::StreamError),
        };

        let stream_clone_one = match stream_lock.try_clone() {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::StreamError),
        };
        let stream_clone_two = match stream_lock.try_clone() {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::StreamError),
        };

        let threadpool = ThreadPool::new(5);

        let subscriptions_clone = self.subscriptions.clone();

        let cloned_self = self.clone();
        let _write_messages = threadpool.execute(move || {
            cloned_self.write_messages(
                stream_clone_one,
                receiver_channel,
                desconectar,
                write_sender,
                subscriptions_clone,
                write_receiver,
            )
        });

        let sender_channel_clone = self.sender_channel.clone();
        let _read_messages = threadpool.execute(move || {
            Client::receive_messages(
                stream_clone_two,
                recieve_receiver,
                reciever_sender,
                sender_channel_clone,
            )
        });

        Ok(())
    }

    pub fn receive_messages(
        stream: TcpStream,
        receiver: Receiver<u16>,
        sender: Sender<bool>,
        sender_channel: Sender<ClientMessage>,
    ) -> Result<(), ProtocolError> {
        let mut pending_messages = Vec::new();

        loop {
            if let Ok(packet) = receiver.try_recv() {
                if !pending_messages.contains(&packet) {
                    pending_messages.push(packet);
                }
            }

            if let Ok(stream_clone) = stream.try_clone() {
                match handle_message(
                    stream_clone,
                    pending_messages.clone(),
                    sender.clone(),
                    sender_channel.clone(),
                ) {
                    Ok(return_val) => {
                        if return_val == ClientReturn::DisconnectRecieved {
                            return Ok(());
                        }
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            } else {
                println!("Error al clonar el stream");
            }
        }
    }

    /// Esta funcion se encarga de la escritura de mensajes que recibe mediante el channel.
    ///
    ///
    /// Si el mensaje es un publish con qos 1, se envia el mensaje y se espera un puback. Si no se recibe, espera 0.5 segundos y reenvia el mensaje. aumentando en 1 el dup_flag, indicando que es al vez numero n que se envia el publish.
    fn write_messages(
        &self,
        stream: TcpStream,
        receiver_channel: Arc<Mutex<Receiver<Box<dyn MessagesConfig + Send>>>>,
        mut desconectar: bool,
        sender: Sender<u16>,
        subscriptions_clone: Arc<Mutex<Vec<String>>>,
        receiver: Receiver<bool>,
    ) -> Result<(), ProtocolError> {
        while !desconectar {
            loop {
                let lock = receiver_channel.lock().unwrap();
                if let Ok(message_config) = lock.recv() {
                    let packet_id = self.assign_packet_id();

                    let message = message_config.parse_message(packet_id);
                    println!("Mensaje a enviar: {:?}", message);

                    match message {
                        ClientMessage::Connect { 0: _ } => todo!(),
                        ClientMessage::Publish {
                            packet_id,
                            topic_name,
                            qos,
                            retain_flag,
                            payload,
                            dup_flag,
                            properties,
                        } => match stream.try_clone() {
                            Ok(stream_clone) => {
                                let publish = ClientMessage::Publish {
                                    packet_id,
                                    topic_name: topic_name.clone(),
                                    qos,
                                    retain_flag,
                                    payload: payload.clone(),
                                    dup_flag,
                                    properties: properties.clone(),
                                };

                                if let Ok(packet_id) = Client::publish_message(
                                    publish,
                                    stream_clone.try_clone().unwrap(),
                                    packet_id,
                                ) {
                                    if qos == 1 {
                                        let stream_clone = stream_clone.try_clone().unwrap();
                                        match sender.send(packet_id) {
                                            Ok(_) => {
                                                if let Ok(puback_recieved) = receiver.try_recv() {
                                                    if !puback_recieved {
                                                        //reenviar el msj con un dup_flag + 1

                                                        thread::sleep(Duration::from_millis(500));
                                                        let publish = ClientMessage::Publish {
                                                            packet_id,
                                                            topic_name,
                                                            qos,
                                                            retain_flag,
                                                            payload,
                                                            dup_flag: dup_flag + 1,
                                                            properties,
                                                        };
                                                        match Client::publish_message(
                                                            publish,
                                                            stream_clone,
                                                            packet_id,
                                                        ) {
                                                            Ok(_) => {
                                                                println!("Reenviando mensaje con dup_flag + 1")
                                                            }
                                                            Err(_) => {
                                                                println!(
                                                                    "Error al reenviar mensaje"
                                                                )
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                println!(
                                                "Error al enviar packet_id de puback al receiver,
                                                error: {}
                                                ",
                                                e)
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                return Err::<(), ProtocolError>(ProtocolError::StreamError);
                            }
                        },
                        ClientMessage::Subscribe {
                            packet_id,
                            properties,
                            payload,
                        } => match stream.try_clone() {
                            Ok(stream_clone) => {
                                let subscribe = ClientMessage::Subscribe {
                                    packet_id,
                                    properties,
                                    payload: payload.clone(),
                                };

                                for p in payload {
                                    if let Ok(stream) = stream_clone.try_clone() {
                                        if let Ok(packet_id) =
                                            Client::subscribe(subscribe.clone(), packet_id, stream)
                                        {
                                            match sender.send(packet_id) {
                                                Ok(_) => {
                                                    let topic_new = p.topic.to_string();
                                                    match subscriptions_clone.lock() {
                                                        Ok(mut guard) => {
                                                            guard.push(topic_new);
                                                        }
                                                        Err(_) => {
                                                            return Err::<(), ProtocolError>(
                                                                ProtocolError::StreamError,
                                                            );
                                                        }
                                                    }
                                                }
                                                Err(_) => {
                                                    return Err::<(), ProtocolError>(
                                                        ProtocolError::StreamError,
                                                    );
                                                }
                                            }
                                        }
                                    } else {
                                        return Err::<(), ProtocolError>(
                                            ProtocolError::StreamError,
                                        );
                                    }
                                }
                            }
                            Err(_) => {
                                return Err::<(), ProtocolError>(ProtocolError::StreamError);
                            }
                        },
                        ClientMessage::Unsubscribe {
                            packet_id,
                            properties,
                            payload,
                        } => match stream.try_clone() {
                            Ok(stream_clone) => {
                                let unsubscribe = ClientMessage::Unsubscribe {
                                    packet_id,
                                    properties,
                                    payload: payload.clone(),
                                };

                                for p in payload {
                                    if let Ok(stream) = stream_clone.try_clone() {
                                        if let Ok(packet_id) = Client::unsubscribe(
                                            unsubscribe.clone(),
                                            stream,
                                            packet_id,
                                        ) {
                                            match sender.send(packet_id) {
                                                Ok(_) => {
                                                    let topic_new = p.topic.to_string();
                                                    match subscriptions_clone.lock() {
                                                        Ok(mut guard) => {
                                                            guard.retain(|x| x != &topic_new);
                                                        }
                                                        Err(_) => {
                                                            return Err::<(), ProtocolError>(
                                                                ProtocolError::StreamError,
                                                            );
                                                        }
                                                    }
                                                }
                                                Err(_) => {
                                                    return Err::<(), ProtocolError>(
                                                        ProtocolError::StreamError,
                                                    );
                                                }
                                            }
                                        }
                                    } else {
                                        return Err::<(), ProtocolError>(
                                            ProtocolError::StreamError,
                                        );
                                    }
                                }
                            }
                            Err(_) => {
                                return Err::<(), ProtocolError>(ProtocolError::StreamError);
                            }
                        },
                        ClientMessage::Disconnect {
                            reason_code: _,
                            session_expiry_interval: _,
                            reason_string: _,
                            client_id,
                        } => {
                            match stream.try_clone() {
                                Ok(stream_clone) => {
                                    let reason = "normal";

                                    let _disconnect = ClientMessage::Disconnect {
                                        reason_code: 0x00,
                                        session_expiry_interval: 0,
                                        reason_string: "Disconnecting".to_string(),
                                        client_id: client_id.clone(),
                                    };

                                    if let Ok(packet_id) =
                                        Client::handle_disconnect(client_id, reason, stream_clone)
                                    {
                                        match sender.send(packet_id) {
                                            Ok(_) => continue,
                                            Err(_) => {
                                                println!(
                                                    "Error al enviar packet_id de puback al receiver"
                                                )
                                            }
                                        }
                                        desconectar = true;
                                        break;
                                    }
                                }
                                Err(_) => {
                                    return Err::<(), ProtocolError>(ProtocolError::StreamError);
                                }
                            }
                            break;
                        }
                        ClientMessage::Pingreq => {
                            match stream.try_clone() {
                                Ok(mut stream_clone) => {
                                    let pingreq = ClientMessage::Pingreq;
                                    match pingreq.write_to(&mut stream_clone) {
                                        //chequear esto
                                        Ok(()) => println!("Pingreq enviado"),
                                        Err(_) => println!("Error al enviar pingreq"),
                                    }
                                }
                                Err(_) => {
                                    return Err::<(), ProtocolError>(ProtocolError::StreamError);
                                }
                            }
                        }
                        ClientMessage::Auth {
                            reason_code: _,
                            authentication_method: _,
                            authentication_data: _,
                            reason_string: _,
                            user_properties: _,
                        } => {
                            println!("auth enviado");
                        }
                    }
                }
            }
        }
        Ok(())
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
        self.client_run()
    }

    fn clone_box(&self) -> Box<dyn ClientTrait> {
        Box::new(self.clone())
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
}

/// Lee del stream un mensaje y lo procesa
/// Devuelve un ClientReturn con informacion del mensaje recibido
/// O ProtocolError en caso de error
pub fn handle_message(
    mut stream: TcpStream,
    pending_messages: Vec<u16>,
    sender: Sender<bool>,
    _sender_chanell_: Sender<ClientMessage>,
) -> Result<ClientReturn, ProtocolError> {
    if let Ok(message) = BrokerMessage::read_from(&mut stream) {
        match message {
            BrokerMessage::Connack {
                session_present: _,
                reason_code: _,
                properties: _,
            } => {
                println!("Recibí un Connack");
                Ok(ClientReturn::ConnackReceived)
            }
            BrokerMessage::Puback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
            } => {
                for pending_message in &pending_messages {
                    let packet_id_bytes: [u8; 2] = pending_message.to_be_bytes();
                    if packet_id_bytes[0] == packet_id_msb && packet_id_bytes[1] == packet_id_lsb {}
                }
                match sender.send(true) {
                    Ok(_) => {
                        println!("puback {:?}", message);
                        Ok(ClientReturn::PubackRecieved)
                    }
                    Err(e) => Err(ProtocolError::ChanellError(e.to_string())),
                }
            }
            BrokerMessage::Disconnect {
                reason_code: _,
                session_expiry_interval: _,
                reason_string,
                user_properties: _,
            } => {
                println!(
                    "Recibí un Disconnect, razon de desconexión: {:?}",
                    reason_string
                );

                Ok(ClientReturn::DisconnectRecieved)
            }
            BrokerMessage::Suback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
            } => {
                for pending_message in &pending_messages {
                    let packet_id_bytes: [u8; 2] = pending_message.to_be_bytes();

                    if packet_id_bytes[0] == packet_id_msb && packet_id_bytes[1] == packet_id_lsb {
                        println!("suback con id {} {} recibido", packet_id_msb, packet_id_lsb);
                    }
                }

                println!("Recibi un mensaje {:?}", message);
                Ok(ClientReturn::SubackRecieved)
            }
            BrokerMessage::PublishDelivery {
                packet_id,
                topic_name: _,
                qos: _,
                retain_flag: _,
                dup_flag: _,
                properties: _,
                payload,
            } => {
                println!(
                    "PublishDelivery con id {} recibido, payload: {:?}",
                    packet_id, payload
                );
                // sender_chanell
                //     .send(ClientMessage::Publish {
                //         packet_id,
                //         topic_name,
                //         qos,
                //         retain_flag,
                //         dup_flag,
                //         properties,
                //         payload,
                //     })
                //     .unwrap();
                Ok(ClientReturn::PublishDeliveryRecieved)
            }
            BrokerMessage::Unsuback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
            } => {
                for pending_message in &pending_messages {
                    let packet_id_bytes: [u8; 2] = pending_message.to_be_bytes();
                    if packet_id_bytes[0] == packet_id_msb && packet_id_bytes[1] == packet_id_lsb {
                        println!(
                            "Unsuback con id {} {} recibido",
                            packet_id_msb, packet_id_lsb
                        );
                    }
                }

                println!("Recibi un mensaje {:?}", message);
                Ok(ClientReturn::UnsubackRecieved)
            }
            BrokerMessage::Pingresp => {
                println!("Recibi un mensaje {:?}", message);
                Ok(ClientReturn::PingrespRecieved)
            }
            BrokerMessage::Auth {
                reason_code: _,
                authentication_method: _,
                authentication_data: _,
                reason_string: _,
                user_properties: _,
            } => {
                println!("recibi un auth!");
                Ok(ClientReturn::AuthRecieved)
            }
        }
    } else {
        Err(ProtocolError::StreamError)
    }
}

#[cfg(test)]

mod tests {

    use std::sync::Condvar;

    use crate::mqtt::broker::Broker;

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
        let args = vec!["127.0.0.1".to_string(), "5028".to_string()];
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
            let address = "127.0.0.1:5028";
            let client = Client::new(rx, address.to_string(), connect_config, tx2).unwrap();
            let packet_id = client.assign_packet_id();
            assert_ne!(packet_id, 0);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_assign_subscription_id() {
        let sub_id = Client::_assign_subscription_id();
        assert_ne!(sub_id, 0);
    }
}
