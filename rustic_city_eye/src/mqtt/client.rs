use rand::Rng;

use std::{
    collections::HashMap, net::TcpStream, sync::{mpsc, Arc, Mutex}
};

use crate::mqtt::{
    broker_message::BrokerMessage,
    client_message::ClientMessage,
    connect_properties::ConnectProperties,
    error::ClientError,
    protocol_error::ProtocolError,
    publish_properties::{PublishProperties, TopicProperties},
    subscribe_properties::SubscribeProperties,
    will_properties::WillProperties,
};

pub struct Client {
    // stream es el socket que se conecta al broker
    stream: TcpStream,
    // las subscriptions son un hashmap de topic y sub_id
    pub subscriptions: Arc<Mutex<Vec<String>>>,
    pending_messages: Vec<u16>,
    // user_id: u32,
}

impl Client {

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        address: String,
        will_properties: WillProperties,
        connect_properties: ConnectProperties,
        clean_start: bool,
        last_will_flag: bool,
        last_will_qos: u8,
        last_will_retain: bool,
        username: String,
        password: String,
        keep_alive: u16,
        client_id: String,
        last_will_topic: String,
        last_will_message: String,
    ) -> Result<Client, ProtocolError> {
        let mut stream = match TcpStream::connect(address) {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::ConectionError),
        };

        let connect = ClientMessage::Connect {
            clean_start,
            last_will_flag,
            last_will_qos,
            last_will_retain,
            username,
            password,
            keep_alive,
            properties: connect_properties,
            client_id,
            will_properties,
            last_will_topic,
            last_will_message,
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

        let user_id = Client::assign_user_id();

        Ok(Client {
            stream,
            subscriptions: Arc::new(Mutex::new(Vec::new())),
            pending_messages: Vec::new(),
        })
    }
    pub fn publish_message(
        message: &str,
        mut stream: TcpStream,
        pending_messages: Vec<u16>,
    ) -> Result<u16, ClientError> {
        let splitted_message: Vec<&str> = message.split(' ').collect();
        //message interface(temp): dup:1 qos:2 retain:1 topic_name:sometopic
        //    let mut dup_flag = false;
        let qos = 1;
        //      let mut retain_flag = false;
        //        let mut packet_id = 0x00;

        // if splitted_message[0] == "dup:1" {
        //     dup_flag = true;
        // }

        // if splitted_message[1] == "qos:1" {
        //     qos = 1;
        //     packet_id = 0x20FF;
        // } else {
        //     dup_flag = false;
        // }

        // if splitted_message[2] == "retain:1" {
        //     retain_flag = true;
        // }

        let topic_properties = TopicProperties {
            topic_alias: 10,
            response_topic: "String".to_string(),
        };

        let packet_id = Client::assign_packet_id(pending_messages);

        let properties = PublishProperties::new(
            1,
            10,
            topic_properties,
            [1, 2, 3].to_vec(),
            "a".to_string(),
            1,
            "a".to_string(),
        );

        let publish = ClientMessage::Publish {
            packet_id,
            topic_name: splitted_message[0].to_string(),
            qos,
            retain_flag: 1,
            payload: "buendia".to_string(),
            dup_flag: 0,
            properties,
        };

        match publish.write_to(&mut stream) {
            Ok(()) => Ok(packet_id),
            Err(_) => Err(ClientError::new("Error al enviar mensaje")),
        }
    }

    pub fn subscribe(
        topic: &str,
        mut stream: TcpStream,
        subscriptions: Arc<Mutex<Vec<String>>>,
        pending_messages: Vec<u16>,
    ) -> Result<u16, ClientError> {
        let packet_id = Client::assign_packet_id(pending_messages);
        let sub_id = Client::assign_subscription_id();
        let topic = topic.to_string();
        
        let subscriptions = subscriptions.clone();

        subscriptions.lock().unwrap().push(topic.clone());
      
        let subscribe = ClientMessage::Subscribe {
            packet_id,
            topic_name: topic.to_string(),
            properties: SubscribeProperties::new(
                sub_id,
                vec![("propiedad".to_string(), "valor".to_string())],
                vec![0, 1, 2, 3],
            ),
        };

        match subscribe.write_to(&mut stream) {
            Ok(()) => Ok(packet_id),
            Err(_) => Err(ClientError::new("Error al enviar mensaje")),
        }

        
    }

    fn assign_subscription_id() -> u32 {
        let mut rng = rand::thread_rng();

        let sub_id: u32 = rng.gen();

        sub_id
    }

    fn assign_user_id() -> u32 {
        //let mut rng = rand::thread_rng();

        //let user_id: u32 = rng.gen();

        1111
    }

    pub fn unsubscribe(
        topic: &str,
        mut stream: TcpStream,
        pending_messages: Vec<u16>,
    ) -> Result<u16, ClientError> {
        let packet_id = Client::assign_packet_id(pending_messages);

        let unsubscribe = ClientMessage::Unsubscribe {
            packet_id,
            topic_name: topic.to_string(),
            properties: SubscribeProperties::new(
                1,
                vec![("propiedad".to_string(), "valor".to_string())],
                vec![0, 1, 2, 3],
            ),
        };

        match unsubscribe.write_to(&mut stream) {
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
    pub fn client_run(&mut self, rx: mpsc::Receiver<String>) -> Result<(), ProtocolError> {
        let (sender, receiver) = mpsc::channel();

        let pending_messages_clone_one = self.pending_messages.clone();
        let pending_messages_clone_two = self.pending_messages.clone();
        let pending_messages_clone_three = self.pending_messages.clone();
        let pending_messages_clone_four = self.pending_messages.clone();


        let stream_clone_one = match self.stream.try_clone() {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::StreamError),
        };
        let stream_clone_two = match self.stream.try_clone() {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::StreamError),
        };
        let stream_clone_three = match self.stream.try_clone() {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::StreamError),
        };
        let stream_clone_five = match self.stream.try_clone() {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::StreamError),
        };

        let subscriptions_clone =  Arc::clone(&self.subscriptions);

        let _write_messages = std::thread::spawn(move || {
            let mut pending_messages = pending_messages_clone_one.clone();
           
            loop {
                if let Ok(line) = rx.recv() {
                    if line.starts_with("publish:") {
                        let (_, post_colon) = line.split_at(8); // "publish:" is 8 characters
                        let message = post_colon.trim(); // remove leading/trailing whitespace
                        println!("Publicando mensaje: {}", message);

                        match stream_clone_one.try_clone() {
                            Ok(stream_clone) => {
                                if let Ok(packet_id) = Client::publish_message(
                                    message,
                                    stream_clone,
                                    pending_messages_clone_one.clone(),
                                ) {
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
                        }
                    } else if line.starts_with("subscribe:") {
                        let (_, post_colon) = line.split_at(10); // "subscribe:" is 10 characters
                        let topic = post_colon.trim(); // remove leading/trailing whitespace
                       
                        println!("Subscribiendome al topic: {}", topic);
                        
                        if subscriptions_clone.lock().unwrap().contains(&topic.to_string()){
                            println!("Ya estoy subscrito a este topic");
                        } else {
                            match stream_clone_two.try_clone() {
                                Ok(stream_clone) => {
                                    if let Ok(packet_id) = Client::subscribe(
                                        topic,
                                        stream_clone,
                                        subscriptions_clone.clone(),
                                        pending_messages_clone_two.clone(),
                                    ) {
                                        match sender.send(packet_id) {
                                            Ok(_) => continue,
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
                        println!("Subscriptions: {:?}", subscriptions_clone);
                    } else if line.starts_with("unsubscribe:") {
                        let (_, post_colon) = line.split_at(12); // "unsubscribe:" is 12 characters
                        let topic = post_colon.trim(); // remove leading/trailing whitespace
                        
                        println!("Desubscribiendome del topic: {}", topic);

                        if !subscriptions_clone.lock().unwrap().contains(&topic.to_string()){
                            println!("No estoy subscrito a este topic");
                        } else {
                            match stream_clone_five.try_clone() {
                                Ok(stream_clone) => {
                                    if let Ok(packet_id) = Client::unsubscribe(
                                        topic,
                                        stream_clone,
                                        pending_messages_clone_three.clone(),
                                    ) {
                                        match sender.send(packet_id) {
                                            Ok(_) => continue,
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
                            }
                        }
                       
                    } else {
                        println!("Comando no reconocido: {}", line);
                    }
                }
            }
        });

        let _read_messages = std::thread::spawn(move || {
            let mut pending_messages = Vec::new();
            
            loop {
                let pending_messages_read = pending_messages_clone_four.clone();
                println!("pending: {:?}", pending_messages_read);
                if let Ok(packet) = receiver.try_recv() {
                    pending_messages.push(packet);
                    println!("pending messages {:?}", pending_messages);
                }

                if let Ok(mut stream_clone) = stream_clone_three.try_clone() {
                    if let Ok(message) = BrokerMessage::read_from(&mut stream_clone) {
                        match message {
                            BrokerMessage::Connack {} => todo!(),
                            BrokerMessage::Puback {
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
                                            "puback con id {} {} recibido",
                                            packet_id_msb, packet_id_lsb
                                        );
                                    }
                                }
                                println!("Recibi un mensaje {:?}", message);
                            }
                            BrokerMessage::Suback {
                                packet_id_msb,
                                packet_id_lsb,
                                reason_code: _,
                            } => {
                                for pending_message in &pending_messages {
                                    let packet_id_bytes: [u8; 2] = pending_message.to_be_bytes();
                                    println!("subscribe scon id {} {} recibido",
                                    packet_id_msb, packet_id_lsb);
                                    if packet_id_bytes[0] == packet_id_msb
                                        && packet_id_bytes[1] == packet_id_lsb
                                    {
                                        println!(
                                            "suback con id {} {} recibido",
                                            packet_id_msb, packet_id_lsb
                                        );
                                    }
                                }
                                println!("Recibi un mensaje {:?}", message);
                            }
                            BrokerMessage::PublishDelivery { payload: _ } => {
                                println!("Recibi un mensaje {:?}", message)
                            }
                            BrokerMessage::Unsuback {
                                packet_id_msb,
                                packet_id_lsb,
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
