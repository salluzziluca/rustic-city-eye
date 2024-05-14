use std::{collections::HashMap, net::{TcpListener, TcpStream}, sync::{Arc, RwLock}};

use crate::mqtt::{
    broker_message::BrokerMessage, protocol_error::ProtocolError, topic::Topic, client_message::ClientMessage

};

static SERVER_ARGS: usize = 2;

#[derive(Clone)]
pub struct Broker {
    address: String,
    topics: Arc<RwLock<HashMap<String, Topic>>>
}

impl Broker {
    ///Chequea que el numero de argumentos sea valido.
    pub fn new(args: Vec<String>) -> Result<Broker, ProtocolError> {
        if args.len() != SERVER_ARGS {
            let app_name = &args[0];
            println!("Usage:\n{:?} <puerto>", app_name);
            return Err(ProtocolError::InvalidNumberOfArguments);
        }

        let address = "127.0.0.1:".to_owned() + &args[1];
        let mut topics = HashMap::new();
        topics.insert("accidente".to_string(), Topic::new());

        Ok(Broker { 
            address,
            topics: Arc::new(RwLock::new(topics))
         })
    }

    /// Ejecuta el servidor.
    /// Crea un enlace en la dirección del broker y, para
    /// cada conexión entrante, crea un hilo para manejar el nuevo cliente.
    pub fn server_run(&mut self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.address)?;
        
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let mut self_clone = self.clone(); // Clone the `self` reference
                    let topics_clone = self.topics.clone();
                    std::thread::spawn(move || {
                        let _ = self_clone.handle_client(stream, topics_clone); // Use the cloned reference
                    });
                }
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }

    ///Se encarga del manejo de los mensajes del cliente. Envia los ACKs correspondientes.
    pub fn handle_client(&mut self, mut stream: TcpStream, topics: Arc<RwLock<HashMap<String, Topic>>>) -> std::io::Result<()> {
        while let Ok(message) = ClientMessage::read_from(&mut stream) {
            match message {
                ClientMessage::Connect {
                    clean_start: _,
                    last_will_flag: _,
                    last_will_qos: _,
                    last_will_retain: _,
                    username: _,
                    password: _,
                    keep_alive: _,
                    properties: _,
                    client_id: _,
                    will_properties: _,
                    last_will_topic: _,
                    last_will_message: _,
                } => {
                    println!("Recibí un connect: {:?}", message);
                    let connack = BrokerMessage::Connack {
                        //session_present: true,
                        //return_code: 0,
                    };
                    println!("Sending connack: {:?}", connack);
                    connack.write_to(&mut stream).unwrap();
                },
                ClientMessage::Publish {
                    packet_id,
                    topic_name: ref topic,
                    qos: _,
                    retain_flag: _,
                    payload,
                    dup_flag: _,
                    properties: _,
                } => {
                    let packet_id_bytes: [u8; 2] = packet_id.to_be_bytes();

                    let mut lock = topics.try_write().unwrap();

                    let delivery_message = BrokerMessage::PublishDelivery { payload };

                    if let Some(topic) = lock.get_mut("accidente") {
                            let _ = topic.send_message(delivery_message);
                    }

                    let puback = BrokerMessage::Puback {
                        packet_id_msb: packet_id_bytes[0],
                        packet_id_lsb: packet_id_bytes[1],
                        reason_code: 1,
                    };
                    puback.write_to(&mut stream)?;

                },
                ClientMessage::Subscribe { packet_id: _, topic_name: _, properties: _ } => {
                    let suback = BrokerMessage::Suback {
                        packet_id_msb: 0,
                        packet_id_lsb: 1,
                        reason_code: 0,
                    };
                    let stream_for_topic = stream.try_clone().unwrap();
                    let mut lock = topics.try_write().unwrap();

                    if let Some(topic) = lock.get_mut("accidente") {
                            topic.add_subscriber(stream_for_topic);
                    }

                    println!("Sending suback: {:?}", suback);
                    match suback.write_to(&mut stream){
                        Ok(_) => println!("suback enviado"),
                        Err(err) => println!("Error al enviar suback: {:?}", err)
                    }
                },
            }
        }

        Ok(())
    }

}