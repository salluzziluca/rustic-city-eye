use rand::Rng;
use std::{
    net::TcpStream,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
};

use crate::{
    mqtt::{
        broker_message::BrokerMessage, client_message::ClientMessage,
        connect_config::ConnectConfig, error::ClientError, messages_config::MessagesConfig,
        protocol_error::ProtocolError,
    },
    utils::threadpool::ThreadPool,
};

use super::client_return::ClientReturn;

#[derive(Debug)]
pub struct Client {
    receiver_channel: Arc<Mutex<Receiver<Box<dyn MessagesConfig + Send>>>>,

    // stream es el socket que se conecta al broker
    stream: TcpStream,

    // las subscriptions es un vector de topics a los que el cliente está subscrito
    pub subscriptions: Arc<Mutex<Vec<String>>>,

    // client_id es el identificador del cliente
    pub client_id: String,
}

impl Client {
    pub fn new(
        receiver_channel: Receiver<Box<dyn MessagesConfig + Send>>,
        address: String,
        connect_config: ConnectConfig,
    ) -> Result<Client, ProtocolError> {
        let mut stream = match TcpStream::connect(address) {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::ConectionError),
        };

        let connect = ClientMessage::Connect {
            clean_start: connect_config.clean_start,
            last_will_flag: connect_config.last_will_flag,
            last_will_qos: connect_config.last_will_qos,
            last_will_retain: connect_config.last_will_retain,
            username: connect_config.username,
            password: connect_config.password,
            keep_alive: connect_config.keep_alive,
            properties: connect_config.properties,
            client_id: connect_config.client_id.clone(),
            will_properties: connect_config.will_properties,
            last_will_topic: connect_config.last_will_topic,
            last_will_message: connect_config.last_will_message,
        };

        println!("Enviando connect message to broker");
        match connect.write_to(&mut stream) {
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
                            println!("todo salio bien!");
                        }
                        _ => {
                            println!("malio sal");
                        }
                    }
                }
                _ => println!("no recibi un Connack :("),
            }
        } else {
            println!("soy el client y no pude leer el mensaje");
        };

        let client_id = connect_config.client_id;

        Ok(Client {
            receiver_channel: Arc::new(Mutex::new(receiver_channel)),
            stream,
            subscriptions: Arc::new(Mutex::new(Vec::new())),
            client_id,
        })
    }

    /// Publica un mensaje en un topic determinado.
    pub fn publish_message(
        message: ClientMessage,
        mut stream: TcpStream,
        packet_id: u16,
    ) -> Result<u16, ClientError> {
        match message.write_to(&mut stream) {
            Ok(()) => Ok(packet_id),
            Err(_) => Err(ClientError::new("Error al enviar mensaje")),
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
        let (sender, receiver) = mpsc::channel();

        let receiver_channel = self.receiver_channel.clone();

        let desconectar = false;

        let stream_clone_one = match self.stream.try_clone() {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::StreamError),
        };
        let stream_clone_two = match self.stream.try_clone() {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::StreamError),
        };

        let threadpool = ThreadPool::new(5);

        let subscriptions_clone = self.subscriptions.clone();

        let _write_messages = threadpool.execute(move || {
            Client::write_messages(
                stream_clone_one,
                receiver_channel,
                desconectar,
                sender,
                subscriptions_clone,
            )
        });

        let _read_messages =
            threadpool.execute(move || Client::receive_messages(stream_clone_two, receiver));

        Ok(())
    }

    pub fn receive_messages(
        stream: TcpStream,
        receiver: Receiver<u16>,
    ) -> Result<(), ProtocolError> {
        let mut pending_messages = Vec::new();

        loop {
            if let Ok(packet) = receiver.try_recv() {
                pending_messages.push(packet);
            }

            if let Ok(stream_clone) = stream.try_clone() {
                match handle_message(stream_clone, pending_messages.clone()) {
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

    fn write_messages(
        stream: TcpStream,
        receiver_channel: Arc<Mutex<Receiver<Box<dyn MessagesConfig + Send>>>>,
        mut desconectar: bool,
        sender: Sender<u16>,
        subscriptions_clone: Arc<Mutex<Vec<String>>>,
    ) -> Result<(), ProtocolError> {
        while !desconectar {
            loop {
                let lock = receiver_channel.lock().unwrap();
                if let Ok(message_config) = lock.recv() {
                    let packet_id = Client::assign_packet_id();

                    let message = message_config.parse_message(packet_id);

                    match message {
                        ClientMessage::Connect {
                            clean_start: _,
                            last_will_flag: _,
                            last_will_qos: _,
                            last_will_retain: _,
                            keep_alive: _,
                            properties: _,
                            client_id: _,
                            will_properties: _,
                            last_will_topic: _,
                            last_will_message: _,
                            username: _,
                            password: _,
                        } => todo!(),
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
                                    topic_name,
                                    qos,
                                    retain_flag,
                                    payload,
                                    dup_flag,
                                    properties,
                                };

                                if let Ok(packet_id) =
                                    Client::publish_message(publish, stream_clone, packet_id)
                                {
                                    match sender.send(packet_id) {
                                        Ok(_) => continue,
                                        Err(_) => {
                                            println!(
                                                "Error al enviar packet_id de puback al receiver"
                                            )
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
                                    if let Ok(packet_id) = Client::subscribe(
                                        subscribe.clone(),
                                        packet_id,
                                        stream_clone.try_clone().unwrap(),
                                    ) {
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
                                    if let Ok(packet_id) = Client::unsubscribe(
                                        unsubscribe.clone(),
                                        stream_clone.try_clone().unwrap(),
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
                                }
                            }
                            Err(_) => {
                                return Err::<(), ProtocolError>(ProtocolError::StreamError);
                            }
                        },
                        ClientMessage::Disconnect {
                            reason_code,
                            session_expiry_interval: _,
                            reason_string: _,
                            client_id,
                        } => {
                            match stream.try_clone() {
                                Ok(stream_clone) => {
                                    let reason = "normal";

                                    let disconnect = ClientMessage::Disconnect {
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

    ///Asigna un id random
    fn assign_packet_id() -> u16 {
        let mut rng = rand::thread_rng();

        let mut packet_id: u16;
        loop {
            packet_id = rng.gen();
            if packet_id != 0 {
                break;
            }
        }
        packet_id
    }
}

/// Lee del stream un mensaje y lo procesa
/// Devuelve un ClientReturn con informacion del mensaje recibido
/// O ProtocolError en caso de error
pub fn handle_message(
    mut stream: TcpStream,
    pending_messages: Vec<u16>,
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
                println!("puback {:?}", message);
                Ok(ClientReturn::PubackRecieved)
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
        }
    } else {
        Err(ProtocolError::StreamError)
    }
}

#[cfg(test)]

mod tests {

    use super::*;

    #[test]
    fn test_assign_packet_id() {
        let packet_id = Client::assign_packet_id();
        assert_ne!(packet_id, 0);
    }

    #[test]
    fn test_assign_subscription_id() {
        let sub_id = Client::_assign_subscription_id();
        assert_ne!(sub_id, 0);
    }
}
