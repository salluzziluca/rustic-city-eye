use rand::Rng;
use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
};

use crate::{
    mqtt::{
        broker_message::BrokerMessage, client_message::ClientMessage,
        connect::connect_config::ConnectConfig, error::ClientError,
        messages_config::MessagesConfig, protocol_error::ProtocolError,
    },
    utils::threadpool::ThreadPool,
};

use super::client_return::ClientReturn;

#[derive(Debug)]
pub struct Client {
    receiver_channel: Arc<Mutex<Receiver<Box<dyn MessagesConfig + Send>>>>,

    // stream es el socket que se conecta al broker
    stream: TcpStream,

    // las subscriptions son un hashmap de topic y sub_id
    pub subscriptions: Arc<Mutex<HashMap<String, u8>>>,
    // user_id: u32,
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

        let connect = ClientMessage::Connect { connect_config };

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

        let _user_id = Client::assign_user_id();

        Ok(Client {
            receiver_channel: Arc::new(Mutex::new(receiver_channel)),
            stream,
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
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
        topic: &str,
        mut stream: TcpStream,
        subscriptions: Arc<Mutex<HashMap<String, u8>>>,
    ) -> Result<u16, ClientError> {
        let sub_id = Client::assign_subscription_id();
        let topic = topic.to_string();

        let subscriptions = subscriptions.clone();

        subscriptions.lock().unwrap().insert(topic.clone(), sub_id);

        match message.write_to(&mut stream) {
            Ok(()) => Ok(packet_id),
            Err(_) => Err(ClientError::new("Error al enviar mensaje")),
        }
    }

    fn assign_subscription_id() -> u8 {
        let mut rng = rand::thread_rng();

        let sub_id: u8 = rng.gen();

        sub_id
    }

    fn assign_user_id() -> u32 {
        let mut rng = rand::thread_rng();

        let user_id: u32 = rng.gen();
        user_id
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

    ///recibe un string que indica la razón de la desconexión y un stream y envia un disconnect message al broker
    /// segun el str reason recibido, modifica el reason_code y el reason_string del mensaje
    ///
    /// devuelve el packet_id del mensaje enviado o un ClientError en caso de error
    pub fn disconnect(reason: &str, mut stream: TcpStream) -> Result<u16, ClientError> {
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
            user_properties: vec![("propiedad".to_string(), "valor".to_string())],
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

        let subscriptions_clone = self.subscriptions.clone();
        let _read_messages = threadpool.execute(move || {
            Client::receive_messages(stream_clone_two, receiver, subscriptions_clone)
        });

        Ok(())
    }

    pub fn receive_messages(
        stream: TcpStream,
        receiver: Receiver<u16>,
        subscriptions_clone: Arc<Mutex<HashMap<String, u8>>>,
    ) -> Result<(), ProtocolError> {
        let mut pending_messages = Vec::new();

        loop {
            if let Ok(packet) = receiver.try_recv() {
                pending_messages.push(packet);
            }

            if let Ok(stream_clone) = stream.try_clone() {
                match handle_message(
                    stream_clone,
                    subscriptions_clone.clone(),
                    pending_messages.clone(),
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

    fn write_messages(
        stream: TcpStream,
        receiver_channel: Arc<Mutex<Receiver<Box<dyn MessagesConfig + Send>>>>,
        mut desconectar: bool,
        sender: Sender<u16>,
        subscriptions_clone: Arc<Mutex<HashMap<String, u8>>>,
    ) -> Result<(), ProtocolError> {
        while !desconectar {
            loop {
                let lock = receiver_channel.lock().unwrap();
                if let Ok(message_config) = lock.recv() {
                    let packet_id = Client::assign_packet_id();

                    let message = message_config.parse_message(packet_id);

                    match message {
                        ClientMessage::Connect { connect_config: _ } => todo!(),
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
                            topic_name,
                            properties,
                        } => {
                            if subscriptions_clone
                                .lock()
                                .unwrap()
                                .contains_key(&topic_name.to_string())
                            {
                                println!("Ya estoy subscrito a este topic");
                            }

                            match stream.try_clone() {
                                Ok(stream_clone) => {
                                    let subscribe = ClientMessage::Subscribe {
                                        packet_id,
                                        topic_name: topic_name.clone(),
                                        properties,
                                    };
                                    if let Ok(packet_id) = Client::subscribe(
                                        subscribe,
                                        packet_id,
                                        &topic_name,
                                        stream_clone,
                                        subscriptions_clone.clone(),
                                    ) {
                                        match sender.send(packet_id) {
                                            Ok(_) => {
                                                let provitional_sub_id = 1;
                                                let topic_new = topic_name.to_string();
                                                subscriptions_clone
                                                    .lock()
                                                    .unwrap()
                                                    .insert(topic_new, provitional_sub_id);
                                            }
                                            Err(_) => {
                                                println!(
                                                    "Error al enviar el packet_id del suback al receiver"
                                                )
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err::<(), ProtocolError>(ProtocolError::StreamError);
                                }
                            }
                        }
                        ClientMessage::Unsubscribe {
                            packet_id,
                            topic_name,
                            properties,
                        } => match stream.try_clone() {
                            Ok(stream_clone) => {
                                let unsubscribe = ClientMessage::Unsubscribe {
                                    packet_id,
                                    topic_name: topic_name.clone(),
                                    properties,
                                };

                                if let Ok(packet_id) =
                                    Client::unsubscribe(unsubscribe, stream_clone, packet_id)
                                {
                                    match sender.send(packet_id) {
                                        Ok(_) => {
                                            let topic_new = topic_name.to_string();
                                            subscriptions_clone.lock().unwrap().remove(&topic_new);
                                        }
                                        Err(_) => {
                                            println!(
                                                    "Error al enviar el packet_id del unsuback al receiver"
                                                )
                                        }
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
                            user_properties: _,
                        } => {
                            let reason = "normal";

                            match stream.try_clone() {
                                Ok(stream_clone) => {
                                    if let Ok(packet_id) = Client::disconnect(reason, stream_clone)
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
    subscriptions_clone: Arc<Mutex<HashMap<String, u8>>>,
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
                sub_id,
            } => {
                for pending_message in &pending_messages {
                    let packet_id_bytes: [u8; 2] = pending_message.to_be_bytes();

                    if packet_id_bytes[0] == packet_id_msb && packet_id_bytes[1] == packet_id_lsb {
                        println!("suback con id {} {} recibido", packet_id_msb, packet_id_lsb);
                    }
                }

                //busca el sub_id 1 en el hash de subscriptions
                //si lo encuentra, lo reemplaza por el sub_id que llega en el mensaje
                let mut topic = String::new();
                let mut lock = subscriptions_clone.lock().unwrap();
                for (key, value) in lock.iter() {
                    if *value == 1 {
                        topic.clone_from(key);
                    }
                }
                lock.remove(&topic);
                lock.insert(topic, sub_id);

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
        let sub_id = Client::assign_subscription_id();
        assert_ne!(sub_id, 0);
    }

    #[test]
    fn test_assign_user_id() {
        let user_id = Client::assign_user_id();
        assert_ne!(user_id, 0);
    }
}
