use std::{
    collections::HashMap,
    io::Error,
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock},
};

use rand::Rng;

use crate::mqtt::{
    broker_message::BrokerMessage, client_message::ClientMessage, protocol_error::ProtocolError,
    topic::Topic,
};

static SERVER_ARGS: usize = 2;

#[derive(Clone)]
pub struct Broker {
    address: String,

    ///Contiene a todos los Topics.
    /// Se identifican con un topic_name unico para cada topic.
    topics: HashMap<String, Topic>,

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

        topics.insert("accidente".to_string(), Topic::new());

        Ok(Broker {
            address,
            topics,
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
        topics: HashMap<String, Topic>,
        //topics: Arc<RwLock<HashMap<String, Vec<TcpStream>>>>,
        packets: Arc<RwLock<HashMap<u16, ClientMessage>>>,
    ) -> std::io::Result<()> {
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
                    topic_name,
                    qos,
                    retain_flag: _,
                    payload,
                    dup_flag: _,
                    properties: _,
                } => {
                    println!("Recibí un Publish");
                    let packet_id = Broker::assign_packet_id(packets.clone());

                    let packet_id_bytes: [u8; 2] = packet_id.to_be_bytes();

                    let reason_code = Broker::handle_publish(payload, topics.clone(), topic_name)?;

                    if qos == 1 {
                        let puback = BrokerMessage::Puback {
                            packet_id_msb: packet_id_bytes[0],
                            packet_id_lsb: packet_id_bytes[1],
                            reason_code,
                        };
                        println!("Enviando un Puback");
                        match puback.write_to(&mut stream) {
                            Ok(_) => println!("Puback enviado"),
                            Err(err) => println!("Error al enviar Puback: {:?}", err),
                        }
                    }
                }
                ClientMessage::Subscribe {
                    packet_id,
                    topic_name,
                    properties: _,
                } => {
                    println!("Recibi un Subscribe");
                    let packet_id_bytes: [u8; 2] = packet_id.to_be_bytes();

                    let suback = BrokerMessage::Suback {
                        packet_id_msb: packet_id_bytes[0],
                        packet_id_lsb: packet_id_bytes[1],
                        reason_code: 0,
                    };
                    let stream_for_topic = match stream.try_clone() {
                        Ok(stream) => stream,
                        Err(err) => {
                            println!("Error al clonar el stream: {:?}", err);
                            return Err(err);
                        }
                    };

                    Broker::handle_subscribe(stream_for_topic, topics.clone(), topic_name);
                    println!("Envío un Suback");
                    match suback.write_to(&mut stream) {
                        Ok(_) => println!("Suback enviado"),
                        Err(err) => println!("Error al enviar suback: {:?}", err),
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_subscribe(stream: TcpStream, mut topics: HashMap<String, Topic>, topic_name: String) {
        if let Some(topic) = topics.get_mut(&topic_name) {
            topic.add_subscriber(stream);
        }
    }

    fn handle_publish(
        payload: String,
        mut topics: HashMap<String, Topic>,
        topic_name: String,
    ) -> Result<u8, Error> {
        if let Some(topic) = topics.get_mut(&topic_name) {
            match topic.deliver_message(payload) {
                Ok(reason_code) => return Ok(reason_code),
                Err(e) => return Err(e),
            };
        }

        Ok(0x80_u8) //Unspecified Error reason code
    }

    ///Asigna un id al packet que ingresa como parametro.
    ///Guarda el packet en el hashmap de paquetes.
    fn assign_packet_id(packets: Arc<RwLock<HashMap<u16, ClientMessage>>>) -> u16 {
        let mut rng = rand::thread_rng();

        let mut packet_id: u16;
        let lock = packets.read().unwrap();
        loop {
            packet_id = rng.gen();
            if packet_id != 0 && !lock.contains_key(&packet_id) {
                break;
            }
        }
        packet_id
    }
}
