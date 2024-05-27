use rand::Rng;
use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
};

use crate::mqtt::{
    broker_message::BrokerMessage, client_message::ClientMessage, connect_config::ConnectConfig,
    error::ClientError, messages_config::MessagesConfig, protocol_error::ProtocolError,
};

#[derive(Debug)]
pub struct Client {
    receiver_channel: Arc<Mutex<Receiver<Box<dyn MessagesConfig + Send>>>>,

    // stream es el socket que se conecta al broker
    stream: TcpStream,
    // las subscriptions son un hashmap de topic y sub_id
    pub subscriptions: Arc<Mutex<HashMap<String, u8>>>,
    // pending_messages es un vector de packet_id
    pending_messages: Vec<u16>,
    // user_id: u32,
}

impl Client {
    #[allow(clippy::too_many_arguments)]
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
            client_id: connect_config.client_id,
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
                   //session_present,
                    //return_code,
                } => {
                    println!("Recibí un Connack");
                },
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
            pending_messages: Vec::new(),
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
        //let mut rng = rand::thread_rng();

        //let user_id: u32 = rng.gen();

        1111
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

        let mut desconectar = false;

        let pending_messages_clone_one = self.pending_messages.clone();

        let stream_clone_one = match self.stream.try_clone() {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::StreamError),
        };
        let stream_clone_two = match self.stream.try_clone() {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::StreamError),
        };

        let subscriptions_clone = self.subscriptions.clone();

        let _write_messages = std::thread::spawn(move || {
            let mut pending_messages = pending_messages_clone_one.clone();
            while !desconectar {
                loop {
                    let lock = receiver_channel.lock().unwrap();
                    if let Ok(message_config) = lock.recv() {
                        let packet_id = Client::assign_packet_id(pending_messages.clone());

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
                            } => match stream_clone_one.try_clone() {
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
                                        pending_messages.push(packet_id);
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

                                match stream_clone_one.try_clone() {
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
                                        return Err::<(), ProtocolError>(
                                            ProtocolError::StreamError,
                                        );
                                    }
                                }
                            }
                            ClientMessage::Unsubscribe {
                                packet_id,
                                topic_name,
                                properties,
                            } => match stream_clone_one.try_clone() {
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
                                                subscriptions_clone
                                                    .lock()
                                                    .unwrap()
                                                    .remove(&topic_new);
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

                                match stream_clone_one.try_clone() {
                                    Ok(stream_clone) => {
                                        if let Ok(packet_id) =
                                            Client::disconnect(reason, stream_clone)
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
                                        return Err::<(), ProtocolError>(
                                            ProtocolError::StreamError,
                                        );
                                    }
                                }
                                break;
                            }

                            ClientMessage::Pingreq => {
                                match stream_clone_one.try_clone() {
                                    Ok(mut stream_clone) => {
                                        let pingreq = ClientMessage::Pingreq;
                                        match pingreq.write_to(&mut stream_clone) {
                                            //chequear esto
                                            Ok(()) => println!("Pingreq enviado"),
                                            Err(_) => println!("Error al enviar pingreq"),
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
                }
            }
            Ok(())
        });

        let subscriptions_clone = self.subscriptions.clone();
        let _read_messages = std::thread::spawn(move || {
            let mut pending_messages = Vec::new();

            loop {
                println!("pending: {:?}", pending_messages);
                if let Ok(packet) = receiver.try_recv() {
                    pending_messages.push(packet);
                    println!("pending messages {:?}", pending_messages);
                }

                if let Ok(mut stream_clone) = stream_clone_two.try_clone() {
                    if let Ok(message) = BrokerMessage::read_from(&mut stream_clone) {
                        match message {
                            BrokerMessage::Connack {} => todo!(),
                            BrokerMessage::Puback {
                                packet_id_msb,
                                packet_id_lsb,
                                reason_code,
                            } => {
                                for pending_message in &pending_messages {
                                    let packet_id_bytes: [u8; 2] = pending_message.to_be_bytes();
                                    if packet_id_bytes[0] == packet_id_msb
                                        && packet_id_bytes[1] == packet_id_lsb
                                    {
                                        println!(
                                            "puback con id {} {} {} recibido",
                                            packet_id_msb, packet_id_lsb, reason_code
                                        );
                                    }
                                }
                                println!("puback {:?}", message);
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
                                break;
                            }
                            BrokerMessage::Suback {
                                packet_id_msb,
                                packet_id_lsb,
                                reason_code: _,
                                sub_id,
                            } => {
                                for pending_message in &pending_messages {
                                    let packet_id_bytes: [u8; 2] = pending_message.to_be_bytes();

                                    if packet_id_bytes[0] == packet_id_msb
                                        && packet_id_bytes[1] == packet_id_lsb
                                    {
                                        println!(
                                            "suback con id {} {} recibido",
                                            packet_id_msb, packet_id_lsb
                                        );
                                    }
                                }

                                //busca el sub_id 1 en el hash de subscriptions
                                //si lo encuentra, lo reemplaza por el sub_id que llega en el mensaje
                                let mut topic = String::new();
                                for (key, value) in subscriptions_clone.lock().unwrap().iter() {
                                    if *value == 1 {
                                        topic.clone_from(key);
                                    }
                                }
                                subscriptions_clone.lock().unwrap().remove(&topic);
                                subscriptions_clone.lock().unwrap().insert(topic, sub_id);

                                println!("Recibi un mensaje {:?}", message);
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
                                    "PublishDelivery con id {} recibido, payload: {}",
                                    packet_id, payload
                                );
                            }
                            BrokerMessage::Unsuback {
                                packet_id_msb,
                                packet_id_lsb,
                                reason_code: _,
                            } => {
                                for pending_message in &pending_messages {
                                    let packet_id_bytes: [u8; 2] = pending_message.to_be_bytes();
                                    if packet_id_bytes[0] == packet_id_msb
                                        && packet_id_bytes[1] == packet_id_lsb
                                    {
                                        println!(
                                            "Unsuback con id {} {} recibido",
                                            packet_id_msb, packet_id_lsb
                                        );
                                    }
                                }

                                println!("Recibi un mensaje {:?}", message);
                            }
                            BrokerMessage::Pingresp => {
                                println!("Recibi un mensaje {:?}", message)
                            }
                        }
                    }
                } else {
                    println!("Error al clonar el stream");
                }
            }
        });
        Ok(())
    }

    ///Asigna un id al packet que ingresa como parametro.
    ///Guarda el packet en el hashmap de paquetes.
    fn assign_packet_id(packets: Vec<u16>) -> u16 {
        let mut rng = rand::thread_rng();

        let mut packet_id: u16;
        loop {
            packet_id = rng.gen();
            if packet_id != 0 && !packets.contains(&packet_id) {
                break;
            }
        }
        packet_id
    }
}
