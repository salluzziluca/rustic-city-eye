use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock},
};

use crate::mqtt::{
    broker_message::BrokerMessage, client_message::ClientMessage, protocol_error::ProtocolError,
};

static SERVER_ARGS: usize = 2;

#[derive(Clone)]
pub struct Broker {
    address: String,
    topics: Arc<RwLock<HashMap<String, Vec<TcpStream>>>>,

    /// El u16 corresponde al packet_id del package, y dentro
    /// de esa clave se guarda el package.
    packets: Arc<RwLock<HashMap<u16, ClientMessage>>>,
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
        let packets = HashMap::new();

        topics.insert("accidente".to_string(), Vec::new());

        Ok(Broker {
            address,
            topics: Arc::new(RwLock::new(topics)),
            packets: Arc::new(RwLock::new(packets)),
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
                    let topics_clone = self.topics.clone();
                    let packets_clone = self.packets.clone();

                    std::thread::spawn(move || {
                        let _ = Broker::handle_client(stream, topics_clone, packets_clone);
                        // Use the cloned reference
                    });
                }
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }

    ///Se encarga del manejo de los mensajes del cliente. Envia los ACKs correspondientes.
    pub fn handle_client(
        mut stream: TcpStream,
        topics: Arc<RwLock<HashMap<String, Vec<TcpStream>>>>,
        _packets: Arc<RwLock<HashMap<u16, ClientMessage>>>,
    ) -> std::io::Result<()> {
        let mut connected = false;
        let start_time = std::time::Instant::now();

        while let Ok(message) = ClientMessage::read_from(&mut stream) {
            if !connected && start_time.elapsed().as_secs() > 10 {
                println!("Tiempo de espera excedido");
                break;
            }
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
                    connected = true;
                    println!("Recibí un Connect");
                    let connack = BrokerMessage::Connack {
                        //session_present: true,
                        //return_code: 0,
                    };
                    println!("Enviando un Connack");
                    match connack.write_to(&mut stream) {
                        Ok(_) => println!("Connack enviado"),
                        Err(err) => println!("Error al enviar Connack: {:?}", err),
                    }
                }
                ClientMessage::Publish {
                    packet_id,
                    topic_name: ref _topic,
                    qos: _,
                    retain_flag: _,
                    payload,
                    dup_flag: _,
                    properties: _,
                } => {
                    println!("Recibí un Publish");
                    // let packet_id = Broker::assign_packet_id(packets.clone());

                    let packet_id_bytes: [u8; 2] = packet_id.to_be_bytes();

                    Broker::handle_publish(payload, topics.clone());

                    let puback = BrokerMessage::Puback {
                        packet_id_msb: packet_id_bytes[0],
                        packet_id_lsb: packet_id_bytes[1],
                        reason_code: 1,
                    };
                    println!("Enviando un Puback");
                    match puback.write_to(&mut stream) {
                        Ok(_) => println!("Puback enviado"),
                        Err(err) => println!("Error al enviar Puback: {:?}", err),
                    }
                }
                ClientMessage::Subscribe {
                    packet_id,
                    topic_name: _,
                    properties: _,
                } => {
                    println!("Recibi un Subscribe");

                    //  let packet_id = Broker::assign_packet_id(packets.clone());
                    let packet_id_bytes: [u8; 2] = packet_id.to_be_bytes();

                    let suback = BrokerMessage::Suback {
                        packet_id_msb: packet_id_bytes[0],
                        packet_id_lsb: packet_id_bytes[1],
                        reason_code: 0,
                    };
                    //let stream_for_topic = stream.try_clone().unwrap();
                    let stream_for_topic = match stream.try_clone() {
                        Ok(stream) => stream,
                        Err(err) => {
                            println!("Error al clonar el stream: {:?}", err);
                            return Err(err);
                        }
                    };

                    Broker::handle_subscribe(stream_for_topic, Arc::clone(&topics));
                    println!("Envío un Suback");
                    match suback.write_to(&mut stream) {
                        Ok(_) => println!("Suback enviado"),
                        Err(err) => println!("Error al enviar suback: {:?}", err),
                    }
                }
                ClientMessage::Disconnect {
                    reason_code: _,
                    session_expiry_interval: _,
                    reason_string,
                    user_properties: _,
                } => {
                    println!(
                        "Recibí un Disconnect, razon de desconexión: {:?}",
                        reason_string
                    );
                }
                ClientMessage::Pingreq => {
                    println!("Recibí un Pingreq");
                    let pingresp = BrokerMessage::Pingresp;
                    println!("Enviando un Pingresp");
                    match pingresp.write_to(&mut stream) {
                        Ok(_) => println!("Pingresp enviado"),
                        Err(err) => println!("Error al enviar Pingresp: {:?}", err),
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_subscribe(stream: TcpStream, topics: Arc<RwLock<HashMap<String, Vec<TcpStream>>>>) {
        let mut lock = match topics.write() {
            Ok(guard) => guard,
            Err(err) => {
                println!("Error al obtener el lock: {:?}", err);
                return;
            }
        };
        if let Some(topic) = lock.get_mut("accidente") {
            topic.push(stream);
        }
    }

    fn handle_publish(payload: String, topics: Arc<RwLock<HashMap<String, Vec<TcpStream>>>>) {
        let lock = topics.read().unwrap();

        if let Some(topic) = lock.get("accidente") {
            let delivery_message = BrokerMessage::PublishDelivery { payload };

            for mut subscriber in topic {
                println!("Enviando un PublishDelivery");
                match delivery_message.write_to(&mut subscriber) {
                    Ok(_) => println!("PublishDelivery enviado"),
                    Err(err) => println!("Error al enviar PublishDelivery: {:?}", err),
                }
            }
        }
    }

    // ///Asigna un id al packet que ingresa como parametro.
    // ///Guarda el packet en el hashmap de paquetes.
    // fn assign_packet_id(packets: Arc<RwLock<HashMap<u16, ClientMessage>>>) -> u16 {
    //     let mut rng = rand::thread_rng();

    //     let mut packet_id: u16;
    //     let lock = packets.read().unwrap();
    //     loop {
    //         packet_id = rng.gen();
    //         if packet_id != 0 && !lock.contains_key(&packet_id) {
    //             break;
    //         }
    //     }
    //     println!("new packet: {:?}", packet_id);
    //     packet_id
    // }
}
