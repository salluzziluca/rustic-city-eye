use std::{
    net::TcpStream, sync::{
        mpsc::{self},
        Arc, Mutex,
    },
};

use crate::mqtt::{
    broker_message::BrokerMessage, 
    client_message::ClientMessage, 
    connect_properties::ConnectProperties, 
    protocol_error::ProtocolError, 
    publish_properties::{PublishProperties, TopicProperties}, 
    subscribe_properties::SubscribeProperties, 
    will_properties::WillProperties
};

static CLIENT_ARGS: usize = 3;

// #[derive(Clone)]
pub struct Client {
    pub(crate) stream: Arc<Mutex<TcpStream>>,
}
#[allow(dead_code)]
impl Client {
    pub fn new(
        args: Vec<String>,
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
        if args.len() != CLIENT_ARGS {
            let app_name = &args[0];
            println!("Usage:\n{:?} <host> <puerto>", app_name);
            return Err(ProtocolError::InvalidNumberOfArguments);
        }

        let address = args[1].clone() + ":" + &args[2];

        let mut stream = match TcpStream::connect(address) {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::ConectionError),
        };

        let will_properties = will_properties;

        let properties = connect_properties;

        let connect = ClientMessage::Connect {
            clean_start,
            last_will_flag,
            last_will_qos,
            last_will_retain,
            username,
            password,
            keep_alive,
            properties,
            client_id,
            will_properties,
            last_will_topic,
            last_will_message,
        };

        println!("Sending connect message to broker");
        connect.write_to(&mut stream).unwrap();

        if let Ok(message) = BrokerMessage::read_from(&mut stream) {
            match message {
                BrokerMessage::Connack {
                   //session_present,
                    //return_code,
                } => {
                    println!("Recibí un connack: {:?}", message);
                },
                _ => println!("no recibi un connack :("),

            }
        } else {
            println!("soy el client y no pude leer el mensaje");
        };

        Ok(Client {
            stream: Arc::new(Mutex::new(stream)),
        })
    }

    pub fn publish_message(message: &str, stream: Arc<Mutex<TcpStream>>) -> Result<u16, ()> {
        let splitted_message: Vec<&str> = message.split(' ').collect();

        //message interface(temp): dup:1 qos:2 retain:1 topic_name:sometopic
        let mut dup_flag = false;
        let mut qos = 0;
        let mut retain_flag = false;
        let mut packet_id = 0x00;

        if splitted_message[0] == "dup:1" {
            dup_flag = true;
        }

        if splitted_message[1] == "qos:1" {
            qos = 1;
            packet_id = 0x20FF;
        } else {
            dup_flag = false;
        }

        if splitted_message[2] == "retain:1" {
            retain_flag = true;
        }

        let topic_properties = TopicProperties {
            topic_alias: 10,
            response_topic: "String".to_string(),
        };

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
            topic_name: splitted_message[3].to_string(),
            qos,
            retain_flag,
            payload: splitted_message[4].to_string(),
            dup_flag,
            properties,
        };
        let _ = message;
        let mut stream = stream.lock().unwrap();
        
        match publish.write_to(&mut *stream) {
            Ok(()) => Ok(packet_id),
            Err(_) => Err(()),
        }

    }

    /// Suscribe al cliente a un topic
    ///
    /// Recibe el nombre del topic al que se quiere suscribir
    /// Creará un mensaje de suscripción y lo enviará al broker
    /// Esperará un mensaje de confirmación de suscripción
    /// Si recibe un mensaje de confirmación, lo imprimirá
    ///
    pub fn subscribe(topic: &str, stream: Arc<Mutex<TcpStream>>) -> Result<u16, ()> {
        let packet_id = 1;

        let subscribe = ClientMessage::Subscribe {
            packet_id,
            topic_name: topic.to_string(),
            properties: SubscribeProperties::new(
                1,
                vec![("propiedad".to_string(), "valor".to_string())],
                vec![0, 1, 2, 3],
            ),
        };

        let mut stream = stream.lock().unwrap();
        
        match subscribe.write_to(&mut *stream) {
            Ok(()) => Ok(packet_id),
            Err(_) => Err(()),
        }        
    }

    /// Se encarga de que el cliente este funcionando correctamente.
    /// El Client debe encargarse de dos tareas: leer mensajes que le lleguen del Broker(ya sean
    /// mensajes ack como puede ser un Connack, Puback, etc, como tambien espera por pubs que vienen
    /// de los topics al que este subscrito). Su segunda tarea es enviar mensajes: puede enviar mensajes
    /// como Publish, Suscribe, etc.
    /// 
    /// Las dos tareas del Client se deben ejecutar concurrentemente, por eso declaramos dos threads(uno de
    /// lectura y otro de escritura), por lo que ambos threads deben compartir el recurso del TcpStream.
    pub fn client_run(&mut self, rx: mpsc::Receiver<String>) -> Result<(), ProtocolError> {
        let stream_reference_one = Arc::clone(&self.stream);
        let stream_reference_two = Arc::clone(&self.stream);
        let stream_reference_three = Arc::clone(&self.stream);
        let (sender, receiver) = mpsc::channel();


        let write_messages = std::thread::spawn(move || {
            loop {
                let _stream_reference = Arc::clone(&stream_reference_one);
                
                if let Ok(line) = rx.recv() {
                    if line.starts_with("publish:") {
                        let (_, post_colon) = line.split_at(8); // "publish:" is 8 characters
                        let message = post_colon.trim(); // remove leading/trailing whitespace
                        println!("Publishing message: {}", message);
                        let _ = match Client::publish_message(message, Arc::clone(&stream_reference_one)) {
                            Ok(packet_id) => sender.send(packet_id),
                            Err(_) => todo!(),
                        };
                        
                    } else if line.starts_with("subscribe:") {
                        let (_, post_colon) = line.split_at(10); // "subscribe:" is 10 characters
                        let topic = post_colon.trim(); // remove leading/trailing whitespace
                        println!("Subscribing to topic: {}", topic);

                        let _ = match Client::subscribe(topic, Arc::clone(&stream_reference_one)) {
                            Ok(packet_id) => sender.send(packet_id),
                            Err(_) => todo!(),
                        };
                    } else {
                        println!("Comando no reconocido: {}", line);
                    }
                }
            }
        });

        let read_ack_messages = std::thread::spawn(move || {
            let mut pending_messages: Vec<u16> = Vec::new();
            loop{
                if let Ok(pending_message) = receiver.recv() {
                    pending_messages.push(pending_message);
                }

                let ack_message = {
                    let stream_reference = Arc::clone(&stream_reference_two);
                    let mut stream = stream_reference.lock().unwrap();
                
                    if let Ok(message) = BrokerMessage::read_from(&mut *stream) {
                        Some(message)
                    } else {
                        None //aca deberiamos intentar levantar el mensaje del topic!
                    } 
                };
      

                if let Some(message) = ack_message {
                    for pending_message in &pending_messages {
                        if message.analize_packet_id(*pending_message) {
                            println!("mensaje {:?}", message);

                            // pending_message.
                        }
                    }
                } 

                // let del_message = {
                //     let stream_reference = Arc::clone(&stream_reference_two);
                //     let mut stream = match stream_reference.lock() {
                //         Ok(stream) => stream,
                //         Err(e) => {print!("Error: {:?}", e); return Err(e)},
                //     };
                //     if let Ok(del_message) = ClientMessage::read_from(&mut *stream) {
                //         Some(del_message)
                //     } else {
                //         None //aca deberiamos intentar levantar el mensaje del topic!
                //     } 
                // };
      

                // if let Some(message) = del_message {
                //     for pending_message in &pending_messages {
                //         if message.analize_packet_id(*pending_message) {
                //             println!("recibi del topic el mensaje {:?}", message);
                //         }
                        
                //     }
                // } 
            }
        });

        let read_delivery_messages = std::thread::spawn(move || {
            loop {
                let del_message = {
                    let stream_reference = Arc::clone(&stream_reference_three);
                    let mut stream = stream_reference.lock().unwrap();
                    let _ = stream.set_nonblocking(true);
                
                    //let stream_reference = Arc::clone(&stream_reference_three);
                    //let mut stream = match stream_reference.lock() {
                    //     Ok(stream) => stream,
                    //     Err(e) => {print!("Error: {:?}", e); return Err(e)},
                    // };
                    if let Ok(del_message) = ClientMessage::read_from(&mut *stream) {
                        Some(del_message)
                    } else {
                        None //aca deberiamos intentar levantar el mensaje del topic!
                    } 
                };
          
    
                if let Some(message) = del_message {
                   println!("recibi del topic el mensaje {:?}", message);
                } 
            }
        });
        Ok(())
    }
}
