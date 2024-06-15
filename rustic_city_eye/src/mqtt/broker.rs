use std::{
    any::Any,
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex, RwLock},
    thread,
    time::Duration,
};

use crate::utils::threadpool::ThreadPool;
use crate::{
    mqtt::{
        broker_message::BrokerMessage,
        client_message::ClientMessage,
        connack_properties::ConnackProperties,
        connect::will_properties,
        payload::Payload,
        protocol_error::ProtocolError,
        protocol_return::ProtocolReturn,
        reason_code::{
            NO_MATCHING_SUBSCRIBERS_HEX, SUB_ID_DUP_HEX, SUCCESS_HEX, UNSPECIFIED_ERROR_HEX,
        },
        topic::Topic,
    },
    utils::payload_types::PayloadTypes,
};

use super::connect::last_will::LastWill;

static SERVER_ARGS: usize = 2;

const THREADPOOL_SIZE: usize = 20;

#[derive(Clone)]
#[allow(dead_code)]
pub struct Broker {
    address: String,

    ///Contiene a todos los Topics.
    /// Se identifican con un topic_name unico para cada topic.
    topics: HashMap<String, Topic>,

    /// El u16 corresponde al packet_id del package, y dentro
    /// de esa clave se guarda el package.
    packets: Arc<RwLock<HashMap<u16, ClientMessage>>>,

    subs: Vec<u32>,

    clients_ids: Arc<RwLock<HashMap<String, Option<LastWill>>>>,

    /// Los clientes se guardan en un HashMap en el cual
    /// las claves son los client_ids, y los valores son
    /// tuplas que contienen el username y password.
    clients_auth_info: HashMap<String, (String, Vec<u8>)>,
}

impl Broker {
    ///Chequea que el numero de argumentos sea valido.
    pub fn new(args: Vec<String>) -> Result<Broker, ProtocolError> {
        if args.len() != SERVER_ARGS {
            let app_name = &args[0];
            println!("Usage:\n{:?} <puerto>", app_name);
            return Err(ProtocolError::InvalidNumberOfArguments);
        }

        let address = "0.0.0.0:".to_owned() + &args[1];

        let topics = Broker::get_broker_starting_topics("./src/monitoring/topics.txt")?;
        let clients_auth_info = Broker::process_clients_file("./src/monitoring/clients.txt")?;

        let packets = HashMap::new();

        Ok(Broker {
            address,
            topics,
            clients_auth_info,
            packets: Arc::new(RwLock::new(packets)),
            subs: Vec::new(),
            clients_ids: Arc::new(HashMap::new().into()),
        })
    }

    pub fn get_broker_starting_topics(
        file_path: &str,
    ) -> Result<HashMap<String, Topic>, ProtocolError> {
        let mut topics = HashMap::new();
        let topic_readings = Broker::process_topic_config_file(file_path)?;

        for topic in topic_readings {
            topics.insert(topic, Topic::new());
        }

        Ok(topics)
    }

    ///Abro y devuelvo las lecturas del archivo de topics.
    fn process_topic_config_file(file_path: &str) -> Result<Vec<String>, ProtocolError> {
        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => return Err(ProtocolError::ReadingTopicConfigFileError),
        };

        let readings = Broker::read_topic_config_file(&file)?;

        Ok(readings)
    }

    ///Devuelvo las lecturas que haga en el archivo de topics.
    fn read_topic_config_file(file: &File) -> Result<Vec<String>, ProtocolError> {
        let reader = BufReader::new(file).lines();
        let mut readings = Vec::new();

        for line in reader {
            match line {
                Ok(line) => readings.push(line),
                Err(_err) => return Err(ProtocolError::ReadingTopicConfigFileError),
            }
        }

        Ok(readings)
    }

    ///Abro y devuelvo las lecturas del archivo de clients.
    pub fn process_clients_file(
        file_path: &str,
    ) -> Result<HashMap<String, (String, Vec<u8>)>, ProtocolError> {
        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => return Err(ProtocolError::ReadingClientsFileError),
        };

        let readings = Broker::read_clients_file(&file)?;

        Ok(readings)
    }

    ///Devuelvo las lecturas que haga en el archivo de clients.
    fn read_clients_file(file: &File) -> Result<HashMap<String, (String, Vec<u8>)>, ProtocolError> {
        let reader = BufReader::new(file).lines();
        let mut readings = HashMap::new();

        for line in reader.map_while(Result::ok) {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 3 {
                let client_id = parts[0].trim().to_string();
                let username = parts[1].trim().to_string();
                let password = parts[2].trim().to_string().into_bytes();
                readings.insert(client_id, (username, password));
            }
        }

        Ok(readings)
    }

    /// Ejecuta el servidor.
    /// Crea un enlace en la dirección del broker y, para
    /// cada conexión entrante, crea un hilo para manejar el nuevo cliente.
    pub fn server_run(&mut self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.address)?;
        println!("Broker escuchando en {}", self.address);
        let threadpool = ThreadPool::new(THREADPOOL_SIZE);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let topics_clone = self.topics.clone();
                    let packets_clone = self.packets.clone();
                    let subs_clone = self.subs.clone();
                    let clients_auth_info_clone = self.clients_auth_info.clone();

                    let client_id = Arc::new(String::new());
                    let (id_sender, id_receiver) = mpsc::channel();
                    threadpool.execute({
                        let mut client_id: Arc<String> = Arc::clone(&client_id);

                        move || loop {
                            if let Ok(id) = id_receiver.try_recv() {
                                let client_id_guard = Arc::make_mut(&mut client_id);
                                *client_id_guard = id;
                                break;
                            }
                        }
                    });

                    let broker = Arc::new(Mutex::new(self.clone()));

                    threadpool.execute({
                        let client_id = Arc::clone(&client_id);
                        let clients_ids_clone = Arc::clone(&self.clients_ids);
                        let clients_ids_clone2 = Arc::clone(&clients_ids_clone);
                        let stream_clone = match stream.try_clone() {
                            Ok(stream) => stream,
                            Err(_) => {
                                return Err(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "Error al clonar el stream",
                                ))
                            }
                        };
                        let broker = Arc::clone(&broker);
                        move || {
                            let result = match Broker::handle_client(
                                stream,
                                topics_clone.clone(),
                                packets_clone,
                                subs_clone,
                                clients_ids_clone,
                                clients_auth_info_clone,
                                id_sender,
                            ) {
                                Ok(_) => Ok(()),
                                Err(err) => {
                                    println!("Error en el hilo del cliente, {:?}", err);
                                    //busco a ver si hay un will message asociado al cliente
                                    if err == ProtocolError::StreamError
                                        || err == ProtocolError::AbnormalDisconnection
                                    {
                                        let client_id_guard = client_id;
                                        let clients_ids_guard: std::sync::RwLockReadGuard<
                                            HashMap<String, Option<LastWill>>,
                                        > = match clients_ids_clone2.read() {
                                            Ok(clients_ids_guard) => clients_ids_guard,
                                            Err(_) => return Err(err),
                                        };
                                        if let Some(will_message) =
                                            clients_ids_guard.get(&*client_id_guard)
                                        {
                                            if let Some(will_message) = will_message {
                                                send_last_will(
                                                    stream_clone,
                                                    will_message,
                                                    topics_clone,
                                                );
                                            }
                                        }
                                    }

                                    Err(err)
                                }
                            };
                            result
                        }
                    });
                }
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }

    ///Se encarga del manejo de los mensajes del cliente. Envia los ACKs correspondientes.
    pub fn handle_client(
        stream: TcpStream,
        topics: HashMap<String, Topic>,
        packets: Arc<RwLock<HashMap<u16, ClientMessage>>>,
        _subs: Vec<u32>,
        clients_ids: Arc<RwLock<HashMap<String, Option<LastWill>>>>,
        clients_auth_info: HashMap<String, (String, Vec<u8>)>,
        id_sender: std::sync::mpsc::Sender<String>,
    ) -> Result<(), ProtocolError> {
        loop {
            let cloned_stream = match stream.try_clone() {
                Ok(stream) => stream,
                Err(_) => return Err(ProtocolError::StreamError),
            };
            match stream.peek(&mut [0]) {
                Ok(_) => {}
                Err(_) => return Err(ProtocolError::AbnormalDisconnection),
            }
            match handle_messages(
                cloned_stream,
                topics.clone(),
                packets.clone(),
                _subs.clone(),
                clients_ids.clone(),
                clients_auth_info.clone(),
                id_sender.clone(),
            ) {
                Ok(return_val) => {
                    if return_val == ProtocolReturn::DisconnectRecieved {
                        return Ok(());
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
    }

    fn handle_subscribe(
        stream: TcpStream,
        mut topics: HashMap<String, Topic>,
        topic_name: String,
        sub_id: u8,
    ) -> Result<u8, ProtocolError> {
        let reason_code;
        if let Some(topic) = topics.get_mut(&topic_name) {
            match topic.add_subscriber(stream, sub_id) {
                0 => {
                    println!("Subscripcion exitosa");
                    reason_code = SUCCESS_HEX;
                }
                0x92 => {
                    println!("SubId duplicado");
                    reason_code = SUB_ID_DUP_HEX;
                }
                _ => {
                    println!("Error no especificado");
                    reason_code = UNSPECIFIED_ERROR_HEX;
                }
            }
        } else {
            reason_code = UNSPECIFIED_ERROR_HEX;
        }

        Ok(reason_code)
    }

    fn handle_publish(
        message: ClientMessage,
        mut topics: HashMap<String, Topic>,
        topic_name: String,
    ) -> Result<u8, ProtocolError> {
        if let Some(topic) = topics.get_mut(&topic_name) {
            match topic.deliver_message(message) {
                Ok(reason_code) => return Ok(reason_code),
                Err(_) => return Err(ProtocolError::PublishError),
            };
        }

        Ok(0x80_u8) //Unspecified Error reason code
    }

    fn handle_unsubscribe(
        mut topics: HashMap<String, Topic>,
        topic_name: String,
        sub_id: u8,
    ) -> Result<u8, ProtocolError> {
        if sub_id == 0 {
            return Ok(NO_MATCHING_SUBSCRIBERS_HEX);
        }

        let reason_code;

        if let Some(topic) = topics.get_mut(&topic_name) {
            match topic.remove_subscriber(sub_id) {
                0 => {
                    println!("Unsubscribe exitoso");
                    reason_code = SUCCESS_HEX;
                }
                _ => {
                    println!("Error no especificado");
                    reason_code = UNSPECIFIED_ERROR_HEX;
                }
            }
        } else {
            println!("Error no especificado");
            reason_code = UNSPECIFIED_ERROR_HEX;
        }
        println!("reason code {:?}", reason_code);

        Ok(reason_code)
    }

    ///Se toma un packet con su respectivo ID y se lo guarda en el hashmap de mensajes que tiene el Broker.
    fn save_packet(
        packets: Arc<RwLock<HashMap<u16, ClientMessage>>>,
        message: ClientMessage,
        packet_id: u16,
    ) {
        let mut lock = match packets.write() {
            Ok(lock) => lock,
            Err(_) => return,
        };

        lock.insert(packet_id, message);
    }
}

///Envia el mensaje de Last Will al cliente.
///
/// Se encarga de la logica necesaria segun los parametros del Last Will y sus properties
///
/// Si hay un delay en el envio del mensaje (delay_interval), se encarga de esperar el tiempo correspondiente.
///
/// Convierte el mensaje en un Publish y lo envia al broker.
fn send_last_will(mut stream: TcpStream, will_message: &LastWill, topics: HashMap<String, Topic>) {
    let properties = will_message.get_properties();
    let interval = properties.get_last_will_delay_interval();
    thread::sleep(std::time::Duration::from_secs(interval as u64));
    let will_topic = will_message.get_topic();
    let message = will_message.get_message();
    let will_qos = will_message.get_qos();
    let will_retain = will_message.get_retain();

    let will_payload = PayloadTypes::WillPayload(message.to_string());

    //publish
    let will_publish = ClientMessage::Publish {
        packet_id: 0,
        topic_name: will_topic.to_string(),
        qos: will_qos as usize,
        retain_flag: will_retain as usize,
        payload: will_payload,
        dup_flag: 0,
        properties: will_message
            .get_properties()
            .clone()
            .to_publish_properties(),
    };
    Broker::handle_publish(will_publish, topics, will_topic.to_string());
}
/// Lee del stream un mensaje y lo procesa
/// Devuelve un ProtocolReturn con informacion del mensaje recibido
/// O ProtocolError en caso de error
pub fn handle_messages(
    mut stream: TcpStream,
    topics: HashMap<String, Topic>,
    packets: Arc<RwLock<HashMap<u16, ClientMessage>>>,
    _subs: Vec<u32>,
    clients_ids: Arc<RwLock<HashMap<String, Option<LastWill>>>>,
    clients_auth_info: HashMap<String, (String, Vec<u8>)>,
    id_sender: std::sync::mpsc::Sender<String>,
) -> Result<ProtocolReturn, ProtocolError> {
    let mensaje = match ClientMessage::read_from(&mut stream) {
        Ok(mensaje) => mensaje,
        Err(_) => return Err(ProtocolError::StreamError),
    };
    match mensaje {
        ClientMessage::Connect { 0: connect } => {
            id_sender.send(connect.client_id.clone());
            println!("Recibí un Connect");

            let connect_clone = connect.clone();
            {
                let clients_ids_read = match clients_ids.read() {
                    Ok(clients_ids_read) => clients_ids_read,
                    Err(_) => return Err(ProtocolError::StreamError),
                };

                if clients_ids_read.contains_key(&connect.client_id) {
                    let disconnect = BrokerMessage::Disconnect {
                        reason_code: 0,
                        session_expiry_interval: 0,
                        reason_string: "El cliente ya está conectado".to_string(),
                        user_properties: Vec::new(),
                    };

                    match disconnect.write_to(&mut stream) {
                        Ok(_) => {
                            println!("Disconnect enviado");
                            return Ok(ProtocolReturn::DisconnectSent);
                        }
                        Err(err) => println!("Error al enviar Disconnect: {:?}", err),
                    }
                }
            }
            let will_message = connect.give_will_message();
            let mut clients_ids_writer: std::sync::RwLockWriteGuard<
                HashMap<String, Option<LastWill>>,
            > = match clients_ids.write() {
                Ok(clients_ids_writer) => clients_ids_writer,
                Err(_) => return Err(ProtocolError::StreamError),
            };
            clients_ids_writer.insert(connect_clone.client_id.clone(), will_message);

            let connack_reason_code = match authenticate_client(
                connect_clone.properties.authentication_method,
                connect_clone.client_id,
                connect_clone.username,
                connect_clone.password,
                clients_auth_info,
            ) {
                Ok(r) => r,
                Err(e) => return Err(e),
            };

            let properties = ConnackProperties {
                session_expiry_interval: 0,
                receive_maximum: 0,
                maximum_packet_size: 0,
                topic_alias_maximum: 0,
                user_properties: Vec::new(),
                authentication_method: "none".to_string(),
                authentication_data: Vec::new(),
                assigned_client_identifier: "none".to_string(),
                maximum_qos: true,
                reason_string: "none".to_string(),
                wildcard_subscription_available: false,
                subscription_identifier_available: false,
                shared_subscription_available: false,
                server_keep_alive: 0,
                response_information: "none".to_string(),
                server_reference: "none".to_string(),
                retain_available: false,
            };

            let connack = BrokerMessage::Connack {
                session_present: false,
                reason_code: connack_reason_code,
                properties,
            };

            println!("Enviando un Connack");
            match connack.write_to(&mut stream) {
                Ok(_) => return Ok(ProtocolReturn::ConnackSent),
                Err(err) => {
                    println!("{:?}", err);
                    return Err(err);
                }
            }
        }
        ClientMessage::Publish {
            packet_id,
            topic_name,
            qos,
            retain_flag,
            payload,
            dup_flag,
            properties,
        } => {
            println!("Recibí un Publish");
            println!("Topic name: {}", topic_name);
            println!("Payload: {:?}", payload);
            let msg = ClientMessage::Publish {
                packet_id,
                topic_name: topic_name.clone(),
                qos,
                retain_flag,
                payload: payload.clone(),
                dup_flag,
                properties,
            };
            Broker::save_packet(packets.clone(), msg.clone(), packet_id);

            let packet_id_bytes: [u8; 2] = packet_id.to_be_bytes();

            let reason_code = Broker::handle_publish(msg, topics.clone(), topic_name)?;

            if qos == 1 && dup_flag == 0 {
                let puback = BrokerMessage::Puback {
                    packet_id_msb: packet_id_bytes[0],
                    packet_id_lsb: packet_id_bytes[1],
                    reason_code,
                };
                println!("Enviando un Puback");
                match puback.write_to(&mut stream) {
                    Ok(_) => {
                        println!("Puback enviado");
                        return Ok(ProtocolReturn::PubackSent);
                    }
                    Err(err) => println!("Error al enviar Puback: {:?}", err),
                }
            } else {
                return Ok(ProtocolReturn::NoAckSent);
            }
        }
        ClientMessage::Subscribe {
            packet_id,
            topic_name,
            properties,
        } => {
            println!("Recibi un Subscribe");
            let msg = ClientMessage::Subscribe {
                packet_id,
                topic_name: topic_name.clone(),
                properties: properties.clone(),
            };
            Broker::save_packet(packets.clone(), msg, packet_id);

            let packet_id_bytes: [u8; 2] = packet_id.to_be_bytes();

            let stream_for_topic = match stream.try_clone() {
                Ok(stream) => stream,
                Err(_) => return Err(ProtocolError::StreamError),
            };

            let reason_code = Broker::handle_subscribe(
                stream_for_topic,
                topics.clone(),
                topic_name,
                properties.sub_id,
            )?;
            match reason_code {
                0 => {
                    println!("Enviando un Suback");
                    let suback = BrokerMessage::Suback {
                        packet_id_msb: packet_id_bytes[0],
                        packet_id_lsb: packet_id_bytes[1],
                        reason_code: 0,
                        sub_id: properties.sub_id,
                    };
                    match suback.write_to(&mut stream) {
                        Ok(_) => {
                            println!("Suback enviado");
                            return Ok(ProtocolReturn::SubackSent);
                        }
                        Err(err) => println!("Error al enviar suback: {:?}", err),
                    }
                }
                _ => {
                    let suback = BrokerMessage::Suback {
                        packet_id_msb: packet_id_bytes[0],
                        packet_id_lsb: packet_id_bytes[1],
                        reason_code: 0x80,
                        sub_id: properties.sub_id,
                    };
                    println!("Enviando un Suback");
                    match suback.write_to(&mut stream) {
                        Ok(_) => {
                            println!("Suback enviado");
                            return Ok(ProtocolReturn::SubackSent);
                        }
                        Err(err) => println!("Error al enviar suback: {:?}", err),
                    }
                }
            }
            return Ok(ProtocolReturn::SubackSent);
        }
        ClientMessage::Unsubscribe {
            packet_id,
            topic_name,
            properties,
        } => {
            println!("Recibí un Unsubscribe");

            let packet_id_bytes: [u8; 2] = packet_id.to_be_bytes();

            let reason_code =
                Broker::handle_unsubscribe(topics.clone(), topic_name, properties.sub_id)?;

            let unsuback = BrokerMessage::Unsuback {
                packet_id_msb: packet_id_bytes[0],
                packet_id_lsb: packet_id_bytes[1],
                reason_code,
            };

            println!("Enviando un Unsuback");
            match unsuback.write_to(&mut stream) {
                Ok(_) => {
                    println!("Unsuback enviado");
                    return Ok(ProtocolReturn::UnsubackSent);
                }
                Err(err) => println!("Error al enviar Unsuback: {:?}", err),
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
            return Ok(ProtocolReturn::DisconnectRecieved);
        }
        ClientMessage::Pingreq => {
            println!("Recibí un Pingreq");
            let pingresp = BrokerMessage::Pingresp;
            println!("Enviando un Pingresp");
            match pingresp.write_to(&mut stream) {
                Ok(_) => {
                    println!("Pingresp enviado");
                    return Ok(ProtocolReturn::PingrespSent);
                }
                Err(err) => println!("Error al enviar Pingresp: {:?}", err),
            }
        }
        ClientMessage::Auth {
            reason_code: _,
            authentication_method: _,
            authentication_data: _,
            reason_string: _,
            user_properties: _,
        } => {
            println!("Recibi un auth");

            return Ok(ProtocolReturn::AuthRecieved);
        }
    }
    Err(ProtocolError::UnspecifiedError)
}

/// Aca se realiza la autenticacion del cliente. Solo se debe llamar apenas llega un packet del tipo
/// Connect.
///
/// Le ingresan como parametros el auth_method(solo vamos a soportar el metodo password-based),
/// el username, client_id y password que vienen definidos en el packet Connect que envia el usuario.
///
/// Devuele en caso exitoso un u8 que representa el reason code del packet Connack que el Broker va a
/// enviarle al Client.
pub fn authenticate_client(
    authentication_method: String,
    client_id: String,
    username: Option<String>,
    password: Option<Vec<u8>>,
    clients_auth_info: HashMap<String, (String, Vec<u8>)>,
) -> Result<u8, ProtocolError> {
    let mut connack_reason_code = 0x00_u8; //success :D

    match authentication_method.as_str() {
        "password-based" => {
            match clients_auth_info.get(&client_id) {
                Some(value) => {
                    if let (Some(username), Some(password)) = (username, password) {
                        if value == &(username, password) {
                            return Ok(connack_reason_code);
                        }
                        connack_reason_code = 0x86_u8; //bad username or password
                    } else {
                        connack_reason_code = 0x86_u8;
                    }
                }
                None => connack_reason_code = 0x85_u8, //client_id not valid
            }
        }
        _ => connack_reason_code = 0x8C_u8,
    }
    Ok(connack_reason_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::{Arc, RwLock};
    use std::thread;

    #[test]
    fn test_01_creating_broker_config_ok() -> std::io::Result<()> {
        let topics = Broker::get_broker_starting_topics("./src/monitoring/topics.txt").unwrap();
        let clients_auth_info =
            Broker::process_clients_file("./src/monitoring/clients.txt").unwrap();

        let mut expected_topics = HashMap::new();
        let mut expected_clients = HashMap::new();

        expected_topics.insert("accidente".to_string(), Topic::new());
        expected_topics.insert("mensajes para juan".to_string(), Topic::new());
        expected_topics.insert("messi".to_string(), Topic::new());
        expected_topics.insert("fulbito".to_string(), Topic::new());
        expected_topics.insert("incidente".to_string(), Topic::new());

        expected_clients.insert(
            "monitoring_app".to_string(),
            (
                "monitoreo".to_string(),
                "monitoreando_la_vida2004".to_string().into_bytes(),
            ),
        );

        expected_clients.insert(
            "camera_system".to_string(),
            (
                "sistema_camaras".to_string(),
                "CamareandoCamaritasForever".to_string().into_bytes(),
            ),
        );

        let topics_to_check = vec![
            "accidente",
            "mensajes para juan",
            "messi",
            "fulbito",
            "incidente",
        ];

        for topic in topics_to_check {
            assert!(topics.contains_key(topic));
        }

        assert_eq!(expected_clients, clients_auth_info);

        Ok(())
    }

    #[test]
    fn test_02_reading_config_files_err() {
        let topics = Broker::get_broker_starting_topics("./aca/estan/los/topics");
        let clients_auth_info = Broker::process_clients_file("./ahperoacavanlosclientesno");

        assert!(topics.is_err());
        assert!(clients_auth_info.is_err());
    }

    #[test]
    fn test_handle_client() {
        // Set up a listener on a local port.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn a thread to simulate a client.
        thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            stream.write_all(b"Hello, world!").unwrap();
        });

        let topics = HashMap::new();
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let subs = vec![];
        let clients_ids = Arc::new(RwLock::new(HashMap::new()));
        let clients_auth_info = HashMap::new();

        // Write a ClientMessage to the stream.
        // You'll need to replace this with a real ClientMessage.
        let mut result: Result<(), ProtocolError> = Err(ProtocolError::UnspecifiedError);

        // Accept the connection and pass the stream to the function.
        if let Ok((stream, _)) = listener.accept() {
            let (id_sender, _) = mpsc::channel();
            // Perform your assertions here
            result = Broker::handle_client(
                stream,
                topics,
                packets,
                subs,
                clients_ids,
                clients_auth_info,
                id_sender,
            );
        }

        // Check that the function returned Ok.
        // You might want to add more checks here, depending on what
        // handle_client is supposed to do.
        assert!(result.is_err());
    }
}
