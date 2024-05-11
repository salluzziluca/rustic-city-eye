use std::{
    net::TcpStream, sync::{mpsc::{self, TryRecvError}, Arc, Mutex}, time::Duration
};

use crate::mqtt::{
    broker_message::BrokerMessage, client_message::ClientMessage, connect_properties::ConnectProperties, protocol_error::ProtocolError, publish_properties::{PublishProperties, TopicProperties}, reader::read_string, subscribe_properties::SubscribeProperties, will_properties::WillProperties
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

    pub fn publish_message(message: &str, stream: Arc<Mutex<TcpStream>>) {
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
        let stream_reference = Arc::clone(&stream);
        let mut stream = stream_reference.lock().unwrap();
        publish.write_to(&mut *stream).unwrap();

        if let Ok(message) = BrokerMessage::read_from(&mut *stream) {
            match message {
                BrokerMessage::Puback {
                    packet_id_msb: _,
                    packet_id_lsb: _,
                    reason_code: _,
                } => {
                    println!("Recibí un puback: {:?}", message);
                }
                _ => println!("no recibi nada :("),
            }
        }
    }

    /// Suscribe al cliente a un topic
    ///
    /// Recibe el nombre del topic al que se quiere suscribir
    /// Creará un mensaje de suscripción y lo enviará al broker
    /// Esperará un mensaje de confirmación de suscripción
    /// Si recibe un mensaje de confirmación, lo imprimirá
    ///
    pub fn subscribe(topic: &str, stream: Arc<Mutex<TcpStream>>) {
        let subscribe = ClientMessage::Subscribe {
            packet_id: 1,
            topic_name: topic.to_string(),
            properties: SubscribeProperties::new(
                1,
                vec![("propiedad".to_string(), "valor".to_string())],
                vec![0, 1, 2, 3],
            ),
        };
    
        let stream_reference = Arc::clone(&stream);
        let mut stream = stream_reference.lock().unwrap();
        
        subscribe.write_to(&mut *stream).unwrap();

        if let Ok(message) = BrokerMessage::read_from(&mut *stream) {
            match message {
                BrokerMessage::Suback {
                    packet_id_msb: _,
                    packet_id_lsb: _,
                    reason_code: _,
                } => {
                    println!("Recibí un suback: {:?}", message);
                }
                _ => println!("Recibí un mensaje que no es suback"),
            }
        } else {
            println!("soy el client y no pude leer el mensaje 2");
        }
    }

    pub fn client_run(&mut self, rx: mpsc::Receiver<String>) -> Result<(), ProtocolError> {
        let stream_reference_one = Arc::clone(&self.stream);
        let stream_reference_two = Arc::clone(&self.stream);

        let handle_receive_messages = std::thread::spawn(move || {
            println!("gola");
            let stream_reference = Arc::clone(&stream_reference_one);
            let mut stream = stream_reference.lock().unwrap();
            loop {
                let message = read_string(&mut *stream).unwrap();
                println!("mensaje {}", message);
            }
        });

        let handle_sending_messages = std::thread::spawn(move || {
            loop {
                match rx.try_recv() {
                    Ok(line) => {
                        if line.starts_with("publish:") {
                            let (_, post_colon) = line.split_at(8); // "publish:" is 8 characters
                            let message = post_colon.trim(); // remove leading/trailing whitespace
                            println!("Publishing message: {}", message);
                            Client::publish_message(message, Arc::clone(&stream_reference_two));
                        } else if line.starts_with("subscribe:") {
                            let (_, post_colon) = line.split_at(10); // "subscribe:" is 10 characters
                            let topic = post_colon.trim(); // remove leading/trailing whitespace
                            println!("Subscribing to topic: {}", topic);
    
                            Client::subscribe(topic, Arc::clone(&stream_reference_two));
                        } else {
                            println!("Comando no reconocido: {}", line);
                        }
                    },
                    Err(TryRecvError::Empty) => {
                        println!("no recibi nati");
                        std::thread::sleep(Duration::from_millis(5000));
                    },
                    Err(TryRecvError::Disconnected) => {
                        println!("Error: el transmisor fue desconectado");
                        break; 
                    }
                }
            }
        });

        


        Ok(())
    }

    // pub fn client_run(&mut self, input_stream: Box<dyn Read + Send>) -> Result<(), ProtocolError> {
    //     let stream_reference = Arc::clone(&self.stream);
    //     let stream_reference1 = Arc::clone(&self.stream);

    //     let input_stream_shared = Arc::new(Mutex::new(input_stream));

    //     let _messages_reception = std::thread::spawn(move || {
    //         println!("toy readi para recibir mensajes");
    //         let mut stream = stream_reference.lock().unwrap();
    //         println!("stream {:?}", stream);
    //         loop {
    //             let mut buf = [0u8; 1];
    //             let numerito = stream.read_exact(&mut buf);
    //             println!("numerito: {:?}", numerito)
    //         }
    //     });

    //     let _message_sender = std::thread::spawn(move || {
    //         let input_stream_ref = Arc::clone(&input_stream_shared);
    //         let mut locked_input = input_stream_ref.lock().unwrap();
    //         let reader = BufReader::new(&mut *locked_input);

    //         for line in reader.lines() {
    //             if let Ok(line) = line {
    //                 println!("line: {:?}", line);
    //                 if line.starts_with("publish:") {
    //                     let (_, post_colon) = line.split_at(8); // "publish:" is 8 characters
    //                     let message = post_colon.trim(); // remove leading/trailing whitespace
    //                     println!("Publishing message: {}", message);
    //                     //Client::publish_message(message, Arc::clone(&stream_reference1));
    //                 } else if line.starts_with("subscribe:") {
    //                     let (_, post_colon) = line.split_at(10); // "subscribe:" is 10 characters
    //                     let topic = post_colon.trim(); // remove leading/trailing whitespace
    //                     println!("Subscribing to topic: {}", topic);

    //                     Client::subscribe(topic, Arc::clone(&stream_reference1));
    //                 } else {
    //                     println!("Comando no reconocido: {}", line);
    //                 }
    //             } else {
    //                 return Err(std::io::Error::new(
    //                     std::io::ErrorKind::Other,
    //                     "Error al leer linea",
    //                 ));
    //             }
    //         }
    //         Ok(())
    //     });
    //     _messages_reception.join().unwrap();
    //     let _ = _message_sender.join().unwrap();
   //     Ok(())
   // }
}
