use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{mpsc, Arc, RwLock},
    thread,
};

use crate::mqtt::{
    broker_message::BrokerMessage,
    client_config::ClientConfig,
    client_message::ClientMessage,
    connack_properties::ConnackProperties,
    protocol_error::ProtocolError,
    protocol_return::ProtocolReturn,
    reason_code::{SUB_ID_DUP_HEX, SUCCESS_HEX, UNSPECIFIED_ERROR_HEX},
    subscription::Subscription,
    topic::Topic,
};

use crate::utils::payload_types::PayloadTypes;
use crate::utils::threadpool::ThreadPool;

use super::{connect::last_will::LastWill, reason_code};

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

    /// Contiene los clientes conectados al broker.

    /// Contiene los clientes desconectados del broker y sus mensajes pendientes.
    offline_clients: Arc<RwLock<HashMap<String, Vec<ClientMessage>>>>,
    #[allow(clippy::type_complexity)]
    clients_ids: Arc<RwLock<HashMap<String, (Option<TcpStream>, Option<LastWill>)>>>,

    /// Los clientes se guardan en un HashMap en el cual
    /// las claves son los client_ids, y los valores son
    /// tuplas que contienen el username y password.
    clients_auth_info: HashMap<String, (String, Vec<u8>)>,

    /// Contiene los clientes conectados al broker.
    clients_config: Arc<RwLock<Vec<(String, ClientConfig)>>>,
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
        let clients_ids = Arc::new(RwLock::new(HashMap::new()));
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let offline_clients = Arc::new(RwLock::new(HashMap::new()));
        let clients_config = Arc::new(RwLock::new(Vec::new()));

        Ok(Broker {
            address,
            topics,
            clients_auth_info,
            packets,
            clients_ids,
            offline_clients,
            clients_config,
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

        let self_clone = self.clone();
        thread::spawn(move || loop {
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => {
                    if input.trim() == "exit" {
                        let _ = self_clone.broker_exit();
                        std::process::exit(0);
                    }
                }
                Err(_) => {}
            }
        });

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let topics_clone = self.topics.clone();
                    let packets_clone = self.packets.clone();
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
                    let self_clone = self.clone();
                    threadpool.execute({
                        let client_id = Arc::clone(&client_id);
                        let clients_ids_clone = Arc::clone(&self.clients_ids);
                        let clients_ids_clone2 = Arc::clone(&clients_ids_clone);
                        let self_ref = Arc::new(self.clone()); // wrap `self` in an Arc

                        move || {
                            let result = match self_clone.handle_client(
                                stream,
                                topics_clone.clone(),
                                packets_clone,
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
                                        #[allow(clippy::type_complexity)]
                                        let clients_ids_guard: std::sync::RwLockReadGuard<
                                            HashMap<String, (Option<TcpStream>, Option<LastWill>)>,
                                        > = match clients_ids_clone2.read() {
                                            Ok(clients_ids_guard) => clients_ids_guard,
                                            Err(_) => return Err(err),
                                        };
                                        if let Some((_, will_message)) =
                                            clients_ids_guard.get(&*client_id_guard)
                                        {
                                            let will_message = match will_message {
                                                Some(will_message) => will_message,
                                                None => return Err(err),
                                            };
                                            let self_clone2 = self_ref.clone();
                                            self_clone2
                                                .clone()
                                                .send_last_will(will_message, topics_clone);
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

    /// Se encarga del manejo de los mensajes del cliente. Envia los ACKs correspondientes.
    #[allow(clippy::type_complexity)]
    pub fn handle_client(
        self,
        stream: TcpStream,
        topics: HashMap<String, Topic>,
        packets: Arc<RwLock<HashMap<u16, ClientMessage>>>,
        clients_ids: Arc<RwLock<HashMap<String, (Option<TcpStream>, Option<LastWill>)>>>,
        clients_auth_info: HashMap<String, (String, Vec<u8>)>,
        id_sender: std::sync::mpsc::Sender<String>,
    ) -> Result<(), ProtocolError> {
        println!("entre a handle client");
        loop {
            let cloned_stream = match stream.try_clone() {
                Ok(stream) => stream,
                Err(_) => {
                    return Err(ProtocolError::StreamError);
                }
            };
            match stream.peek(&mut [0]) {
                Ok(_) => {}
                Err(_) => return Err(ProtocolError::AbnormalDisconnection),
            }
            match self.handle_messages(
                cloned_stream,
                topics.clone(),
                packets.clone(),
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

    /// Convierte un mensaje del cliente a un mensaje del broker.
    ///
    /// Retorna el mensaje del broker si la conversion fue exitosa, o un error si no lo fue.
    fn convert_to_broker_message(message: &ClientMessage) -> Result<BrokerMessage, ProtocolError> {
        match message {
            ClientMessage::Publish {
                packet_id,
                topic_name,
                qos,
                retain_flag,
                payload,
                dup_flag,
                properties,
            } => Ok(BrokerMessage::PublishDelivery {
                packet_id: *packet_id,
                topic_name: topic_name.clone(),
                qos: *qos,
                retain_flag: *retain_flag,
                payload: payload.clone(),
                dup_flag: *dup_flag,
                properties: properties.clone(),
            }),
            _ => Err(ProtocolError::UnspecifiedError),
        }
    }

    /// Envia un mensaje a un usuario.
    ///
    /// Retorna Ok si el mensaje fue enviado, Err si el usuario está offline.
    fn send_message_to_user(
        &self,
        user: &Subscription,
        message: &BrokerMessage,
    ) -> Result<(), bool> {
        let clients = self.clients_ids.read().map_err(|_| false)?;
        if let Some((stream, _)) = clients.get(&user.client_id) {
            if let Some(stream) = stream {
                let mut stream_clone = stream.try_clone().expect("Error al clonar el stream");
                message.write_to(&mut stream_clone).map_err(|_| true)?;
                println!("Mensaje enviado a {}", user.client_id);
                Ok(())
            } else {
                Err(true)
            }
        } else {
            Err(true)
        }
    }

    /// Maneja el mensaje de un usuario offline.
    ///
    /// Guarda el mensaje en offline_clients.
    fn handle_offline_user(
        &self,
        user_id: &str,
        message: &ClientMessage,
    ) -> Result<(), ProtocolError> {
        let mut offline_clients = self
            .offline_clients
            .write()
            .map_err(|_| ProtocolError::UnspecifiedError)?;
        if let Some(messages) = offline_clients.get_mut(user_id) {
            messages.push(message.clone());
        } else {
            offline_clients.insert(user_id.to_string(), vec![message.clone()]);
        }
        Ok(())
    }

    /// Maneja el envio de un mensaje a un topic.
    ///
    /// Retorna el reason code correspondiente a si el envio fue exitoso o no.
    fn handle_publish(
        &self,
        message: ClientMessage,
        mut topics: HashMap<String, Topic>,
        topic_name: String,
    ) -> Result<u8, ProtocolError> {
        let mensaje = Broker::convert_to_broker_message(&message)?;
        let topic = topics
            .get_mut(&topic_name)
            .ok_or(ProtocolError::UnspecifiedError)?;
        let users = topic.get_users_from_topic();

        for user in users {
            match self.send_message_to_user(&user, &mensaje) {
                Ok(_) => (),
                Err(esta_offline) => {
                    if esta_offline {
                        self.handle_offline_user(&user.client_id, &message)?;
                    }
                }
            }
        }
        Ok(0x80_u8) // Unspecified Error reason code
    }

    /// Maneja la subscripcion de un cliente a un topic.
    /// Devuelve el reason code correspondiente a si la subscripcion fue exitosa o no.
    /// Si el reason code es 0, el cliente se ha suscrito exitosamente.
    fn handle_subscribe(
        mut topics: HashMap<String, Topic>,
        topic_name: String,
        subscription: Subscription,
    ) -> Result<u8, ProtocolError> {
        
        let reason_code;       
        if let Some(topic) = topics.get_mut(&topic_name) {
            match topic.add_user_to_topic(subscription.clone()) {
                0 => {
                    ClientConfig::add_new_subscription(subscription.client_id.clone(), topic_name.clone());
                    reason_code = SUCCESS_HEX;
                }
                0x92 => {
                    reason_code = SUB_ID_DUP_HEX;
                }
                _ => {
                    reason_code = UNSPECIFIED_ERROR_HEX;
                }
            }
        } else {
            reason_code = UNSPECIFIED_ERROR_HEX;
        }
            
        Ok(reason_code)
    }

    /// Maneja la desubscripcion de un cliente a un topic
    /// Devuelve el reason code correspondiente a si la desubscripcion fue exitosa o no
    /// Si el reason code es 0, el cliente se ha desuscrito exitosamente.
    fn handle_unsubscribe(
        mut topics: HashMap<String, Topic>,
        topic_name: String,
        subscription: Subscription,
    ) -> Result<u8, ProtocolError> {
        let reason_code;

        if let Some(topic) = topics.get_mut(&topic_name) {
            match topic.remove_user_from_topic(subscription.clone()) {
                0 => {
                    println!("Unsubscribe exitoso");
                    ClientConfig::remove_subscription(subscription.client_id.clone(), topic_name.clone());
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

    /// Envia el mensaje de Last Will al cliente.
    ///
    /// Se encarga de la logica necesaria segun los parametros del Last Will y sus properties
    ///
    /// Si hay un delay en el envio del mensaje (delay_interval), se encarga de esperar el tiempo correspondiente.
    ///
    /// Convierte el mensaje en un Publish y lo envia al broker.
    fn send_last_will(&self, will_message: &LastWill, topics: HashMap<String, Topic>) {
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
            topic_name: will_topic.to_string(), //TODO: aca habria que ver bien cual topic le cargamos
            qos: will_qos as usize,
            retain_flag: will_retain as usize,
            payload: will_payload,
            dup_flag: 0,
            properties: will_message
                .get_properties()
                .clone()
                .to_publish_properties(),
        };
        _ = self.handle_publish(will_publish, topics, will_topic.to_string());
    }

    /// Lee del stream un mensaje y lo procesa
    /// Devuelve un ProtocolReturn con informacion del mensaje recibido
    /// O ProtocolError en caso de erro    #[allow(clippy::type_complexity)]
    #[allow(clippy::type_complexity)]
    pub fn handle_messages(
        &self,
        mut stream: TcpStream,
        topics: HashMap<String, Topic>,
        packets: Arc<RwLock<HashMap<u16, ClientMessage>>>,
        _clients_ids: Arc<RwLock<HashMap<String, (Option<TcpStream>, Option<LastWill>)>>>,
        clients_auth_info: HashMap<String, (String, Vec<u8>)>,
        _id_sender: std::sync::mpsc::Sender<String>,
    ) -> Result<ProtocolReturn, ProtocolError> {
        let mensaje = match ClientMessage::read_from(&mut stream) {
            Ok(mensaje) => mensaje,
            Err(e) => {
                println!("Error al leer mensaje: {:?}", e);
                return Err(ProtocolError::StreamError);
            }
        };

        match mensaje {
            ClientMessage::Connect { 0: connect } => {
                println!("Recibí un Connect");

                // si el cliente ya está conectado, no permite la nueva conexión y la rechaza con CLIENT_DUP
                match self.clients_config.read() {
                    Ok(clients_config) => {
                        for client in clients_config.iter() {
                            if client.1.client_id == connect.client_id {
                                let disconnect = BrokerMessage::Disconnect {
                                    reason_code: 0,
                                    session_expiry_interval: 0,
                                    reason_string: "CLIENT_DUP".to_string(),
                                    user_properties: Vec::new(),
                                };
                                match disconnect.write_to(&mut stream) {
                                    Ok(_) => {
                                        println!("Disconnect enviado");
                                        ClientConfig::change_client_state(
                                            client.1.client_id.clone(),
                                            false,
                                        );
                                        return Ok(ProtocolReturn::DisconnectSent);
                                    }
                                    Err(err) => println!("Error al enviar Disconnect: {:?}", err),
                                }
                            }
                        }
                    }
                    Err(_) => println!("Error al leer clients_config"),
                }

                //
                //     match self.clients_ids.read() {
                //         Ok(clients) => {
                //             if clients.contains_key(&connect.client_id) {
                //                 let disconnect = BrokerMessage::Disconnect {
                //                     reason_code: 0,
                //                     session_expiry_interval: 0,
                //                     reason_string: "CLIENT_DUP".to_string(),
                //                     user_properties: Vec::new(),
                //                 };
                //                 match disconnect.write_to(&mut stream) {
                //                     Ok(_) => {
                //                         println!("Disconnect enviado");
                //                         return Ok(ProtocolReturn::DisconnectSent);
                //                     }
                //                     Err(err) => println!("Error al enviar Disconnect: {:?}", err),
                //                 }
                //             }
                //         }
                //         Err(_) => {
                //             // Manejo de error al leer clients_ids
                //         }
                //     }

                // if let Ok(offline_clients) = self.offline_clients.read() {
                //     if offline_clients.contains_key(&connect.client_id) {
                //         if let Some(pending_messages) = offline_clients.get(&connect.client_id) {
                //             for message in pending_messages {
                //                 match message.write_to(&mut stream) {
                //                     Ok(_) => {
                //                         println!("Mensaje enviado a {}", connect.client_id);
                //                     }
                //                     Err(_) => return Err(ProtocolError::UnspecifiedError),
                //                 }
                //             }
                //         }
                //     }
                // }

                // reibe los mensajes de cuando estuvo offline
                // verifica si el state del cliente es false, si es asi, le envia los mensajes que tenia pendientes

                //si está en offline_clients lo elimino de ahí
                if let Ok(mut lock) = self.offline_clients.write() {
                    lock.remove(&connect.client_id);
                } else {
                    return Err(ProtocolError::UnspecifiedError);
                }

                //clona stream con ok err
                let cloned_stream = match stream.try_clone() {
                    Ok(stream) => stream,
                    Err(_) => return Err(ProtocolError::StreamError),
                };

                let will_message = connect.clone().give_will_message();
                if let Ok(mut clients) = self.clients_ids.write() {
                    ClientConfig::save_client_log_in_json(connect.client_id.clone());
                    clients.insert(
                        connect.client_id.clone(),
                        (Some(cloned_stream), will_message),
                    );
                } else {
                    return Err(ProtocolError::UnspecifiedError);
                }

                let connect_clone = connect.clone();
                let _connack_reason_code = match authenticate_client(
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
                    reason_code: 0,
                    properties,
                };
                println!("Enviando un Connack");
                match connack.write_to(&mut stream) {
                    Ok(_) => return Ok(ProtocolReturn::ConnackSent),
                    Err(err) => {
                        println!("{:?}", err);
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

                let reason_code = self.handle_publish(msg, topics.clone(), topic_name)?;
                if qos == 1 {
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
                properties,
                payload,
            } => {
                println!("Recibí un Subscribe");
                let msg = ClientMessage::Subscribe {
                    packet_id,
                    properties: properties.clone(),
                    payload: payload.clone(),
                };
                Broker::save_packet(packets.clone(), msg, packet_id);

                let packet_id_bytes: [u8; 2] = packet_id.to_be_bytes();

                let reason_code =
                    Broker::handle_subscribe(topics.clone(), payload.topic.clone(), payload.clone())?;

                

                let suback = BrokerMessage::Suback {
                    packet_id_msb: packet_id_bytes[0],
                    packet_id_lsb: packet_id_bytes[1],
                    reason_code,
                };
                println!("Enviando un Suback");
                match suback.write_to(&mut stream) {
                    Ok(_) => {
                        println!("Suback enviado");
                        return Ok(ProtocolReturn::SubackSent);
                    }
                    Err(err) => println!("Error al enviar suback: {:?}", err),
                }

                return Ok(ProtocolReturn::SubackSent);
            }
            ClientMessage::Unsubscribe {
                packet_id,
                properties,
                payload,
            } => {
                println!("Recibí un Unsubscribe");
                let msg = ClientMessage::Unsubscribe {
                    packet_id,
                    properties: properties.clone(),
                    payload: payload.clone(),
                };
                Broker::save_packet(packets.clone(), msg, packet_id);

                let packet_id_bytes: [u8; 2] = packet_id.to_be_bytes();

            
                let reason_code =
                        Broker::handle_unsubscribe(topics.clone(), payload.topic.clone(), payload)?;
                   
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
                client_id,
            } => {
                println!(
                    "Recibí un Disconnect, razon de desconexión: {:?}",
                    reason_string
                );

                if reason_string == "CLIENT_DUP" {
                    return Ok(ProtocolReturn::DisconnectRecieved);
                }

                // elimino el client_id de clients_ids
                if let Ok(mut lock) = self.clients_ids.write() {
                    lock.remove(&client_id);
                } else {
                    return Err(ProtocolError::UnspecifiedError);
                }

                // cambio el estado del cliente a desconectado
                ClientConfig::change_client_state(client_id.clone(), false);

                // agrego el client_id a offline_clients
                if let Ok(mut lock) = self.offline_clients.write() {
                    lock.insert(client_id, Vec::new());
                } else {
                    return Err(ProtocolError::UnspecifiedError);
                }

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
                authentication_method,
                authentication_data,
                reason_string,
                user_properties,
            } => {
                println!("Recibi un auth");

                match authentication_method.as_str() {
                    "password-based" => return Ok(ProtocolReturn::AuthRecieved),
                    _ => {
                        let properties = ConnackProperties {
                            session_expiry_interval: 0,
                            receive_maximum: 0,
                            maximum_packet_size: 0,
                            topic_alias_maximum: 0,
                            user_properties,
                            authentication_method,
                            authentication_data,
                            assigned_client_identifier: "none".to_string(),
                            maximum_qos: true,
                            reason_string,
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
                            reason_code: 0x8C, //Bad auth method
                            properties,
                        };
                        println!("Parece que intentaste autenticarte con un metodo no soportado por el broker :(");

                        match connack.write_to(&mut stream) {
                            Ok(_) => return Ok(ProtocolReturn::ConnackSent),
                            Err(err) => {
                                println!("{:?}", err);
                            }
                        }
                    }
                }
            }
        }
        Err(ProtocolError::UnspecifiedError)
    }

    /// Devuelve los clientes offline y sus mensajes pendientes de manera estática
    /// para poder testear
    pub fn get_offline_clients(&self) -> HashMap<String, Vec<ClientMessage>> {
        match self.offline_clients.read() {
            Ok(lock) => lock.clone(),
            Err(_) => HashMap::new(),
        }
    }

    /// Devuelve los clientes conectados de manera estática
    /// para poder testear
    pub fn get_clients_ids(&self) -> Vec<String> {
        let mut clients_ids = Vec::new();
        let lock = match self.clients_ids.read() {
            Ok(lock) => lock,
            Err(_) => return clients_ids,
        };
        for client_id in lock.keys() {
            // agrego el client_id al vector
            clients_ids.push(client_id.clone());
        }

        clients_ids
    }

    pub fn broker_exit(&self) -> Result<(), ProtocolError> {
        let clients = self
            .clients_ids
            .read()
            .map_err(|_| ProtocolError::UnspecifiedError)?;
        for (client_id, (stream, _)) in clients.iter() {
            if let Some(stream) = stream {
                let mut stream_clone = stream.try_clone().expect("Error al clonar el stream");
                let disconnect = BrokerMessage::Disconnect {
                    reason_code: 0,
                    session_expiry_interval: 0,
                    reason_string: "SERVER_SHUTDOWN".to_string(),
                    user_properties: Vec::new(),
                };
                match disconnect.write_to(&mut stream_clone) {
                    Ok(_) => {
                        println!("Disconnect enviado");
                        ClientConfig::change_client_state(client_id.clone(), false);
                    }
                    Err(_) => return Err(ProtocolError::UnspecifiedError),
                }
            }
        }

        Ok(())
    }
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

    // #[test]
    // fn test_01_creating_broker_config_ok() -> std::io::Result<()> {
    //     let topics = match Broker::get_broker_starting_topics("./src/monitoring/topics.txt") {
    //         Ok(topics) => topics,
    //         Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Error")),
    //     };
    //     let clients_auth_info = match Broker::process_clients_file("./src/monitoring/clients.txt") {
    //         Ok(clients_auth_info) => clients_auth_info,
    //         Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Error")),
    //     };

    //     let mut expected_topics = HashMap::new();
    //     let mut expected_clients = HashMap::new();

    //     expected_topics.insert("accidente".to_string(), Topic::new());
    //     expected_topics.insert("mensajes para juan".to_string(), Topic::new());
    //     expected_topics.insert("messi".to_string(), Topic::new());
    //     expected_topics.insert("fulbito".to_string(), Topic::new());
    //     expected_topics.insert("incidente".to_string(), Topic::new());

    //     expected_clients.insert(
    //         "monitoring_app".to_string(),
    //         (
    //             "monitoreo".to_string(),
    //             "monitoreando_la_vida2004".to_string().into_bytes(),
    //         ),
    //     );

    //     expected_clients.insert(
    //         "camera_system".to_string(),
    //         (
    //             "sistema_camaras".to_string(),
    //             "CamareandoCamaritasForever".to_string().into_bytes(),
    //         ),
    //     );

    //     let topics_to_check = vec![
    //         "accidente",
    //         "mensajes para juan",
    //         "messi",
    //         "fulbito",
    //         "incidente",
    //     ];

    //     for topic in topics_to_check {
    //         assert!(topics.contains_key(topic));
    //     }

    //     assert_eq!(expected_clients, clients_auth_info);

    //     Ok(())
    // }

    #[test]
    fn test_02_reading_config_files_err() {
        let topics: Result<HashMap<String, Topic>, ProtocolError> =
            Broker::get_broker_starting_topics("./aca/estan/los/topics");
        let clients_auth_info = Broker::process_clients_file("./ahperoacavanlosclientesno");

        assert!(topics.is_err());
        assert!(clients_auth_info.is_err());
    }

    #[test]
    fn test_handle_client() {
        // Set up a listener on a local port.
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => listener,
            Err(_) => return,
        };
        let addr = match listener.local_addr() {
            Ok(addr) => addr,
            Err(_) => return,
        };

        // Spawn a thread to simulate a client.
        thread::spawn(move || {
            let mut stream = match TcpStream::connect(addr) {
                Ok(stream) => stream,
                Err(_) => return,
            };
            if stream.write_all(b"Hello, world!").is_ok() {}
        });
        let packets = Arc::new(RwLock::new(HashMap::new()));
        let clients_ids = Arc::new(RwLock::new(HashMap::new()));
        let clients_auth_info = HashMap::new();
        let topics = HashMap::new();
        // Write a ClientMessage to the stream.
        // You'll need to replace this with a real ClientMessage.
        let mut result: Result<(), ProtocolError> = Err(ProtocolError::UnspecifiedError);
        let broker = match Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]) {
            Ok(broker) => broker,
            Err(_) => return,
        };

        // Accept the connection and pass the stream to the function.
        if let Ok((stream, _)) = listener.accept() {
            let (id_sender, _) = mpsc::channel();
            // Perform your assertions here
            result = broker.handle_client(
                stream,
                topics,
                packets,
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
