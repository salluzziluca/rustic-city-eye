use std::{
    collections::HashMap,
    fs::File,
    io::{stdin, BufRead, BufReader},
    net::{Shutdown, TcpListener, TcpStream},
    process::exit,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex, RwLock,
    },
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

use super::connect::last_will::LastWill;

static SERVER_ARGS: usize = 2;

const THREADPOOL_SIZE: usize = 30;

#[derive(Clone)]
pub struct Broker {
    address: String,

    ///Contiene a todos los Topics.
    /// Se identifican con un topic_name unico para cada topic.
    topics: HashMap<String, Topic>,

    /// El u16 corresponde al packet_id del package, y dentro
    /// de esa clave se guarda el package.
    packets: Arc<RwLock<HashMap<u16, ClientMessage>>>,

    /// Contiene los clientes conectados al broker.
    #[allow(clippy::type_complexity)]
    clients_ids: Arc<RwLock<HashMap<String, (Option<Arc<TcpStream>>, Option<LastWill>)>>>,

    /// Los clientes se guardan en un HashMap en el cual
    /// las claves son los client_ids, y los valores son
    /// tuplas que contienen el username y password.
    clients_auth_info: HashMap<String, (String, Vec<u8>)>,
}

impl Broker {
    ///Chequea que el numero de argumentos sea valido.
    pub fn new(args: Vec<String>) -> Result<Broker, ProtocolError> {
        let address = Broker::process_starting_args(args)?;

        let topics = Broker::get_broker_starting_topics("./src/monitoring/topics.txt")?;
        let clients_auth_info = Broker::process_clients_file("./src/monitoring/clients.txt")?;
        let clients_ids = Arc::new(RwLock::new(HashMap::new()));
        let packets = Arc::new(RwLock::new(HashMap::new()));

        Ok(Broker {
            address,
            topics,
            clients_auth_info,
            packets,
            clients_ids,
        })
    }

    fn process_starting_args(args: Vec<String>) -> Result<String, ProtocolError> {
        if args.len() != SERVER_ARGS {
            let app_name = &args[0];
            println!("Usage:\n{:?} <puerto>", app_name);
            return Err(ProtocolError::InvalidNumberOfArguments);
        }

        let address = "0.0.0.0:".to_owned() + &args[1];
        Ok(address)
    }

    /// Recibe un path a un archivo de configuracion de topics y devuelve un HashMap con los topics.
    fn get_broker_starting_topics(
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
    fn process_clients_file(
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

    /// Intenta crear el binding en el address indicado. Retorna un ProtocolError en caso de fallar.
    fn bind_to_address(address: &str) -> Result<TcpListener, ProtocolError> {
        match TcpListener::bind(address) {
            Ok(listener) => {
                println!("Broker escuchando en {}", address);
                Ok(listener)
            }
            Err(e) => Err(ProtocolError::BindingError(e.to_string())),
        }
    }

    /// Ejecuta el servidor.
    /// Crea un enlace en la dirección del broker y, para
    /// cada conexión entrante, crea un hilo para manejar el nuevo cliente.
    pub fn server_run(&mut self) -> Result<(), ProtocolError> {
        let listener = Broker::bind_to_address(&self.address)?;

        let threadpool = ThreadPool::new(THREADPOOL_SIZE);

        let broker_ref = Arc::new(Mutex::new(self.clone()));

        threadpool.execute(move || loop {
            let mut lock = match broker_ref.lock() {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Error obteniendo el lock: {}", e);
                    return;
                }
            };
            let stdin = stdin().lock();

            match lock.process_input_command(stdin) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            }
        });

        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    //let topics_clone = self.topics.clone();
                    let self_clone = self.clone();

                    //let client_id = Arc::new(String::new());
                    let stream = Arc::new(stream);
                    let stream_write_half = Arc::clone(&stream);
                    let stream_read_half = Arc::clone(&stream);

                    let (message_to_write_sender, message_to_write_receiver) = mpsc::channel();
                    let (stream_error_notifier_sender, _stream_error_notifier_receiver) =
                        mpsc::channel();
                    let (disconnect_notifier_sender, disconnect_notifier_receiver) = mpsc::channel();

                    let _handle_read_messages = threadpool.execute(move || loop {
                        let stream_ref = Arc::clone(&stream);

                        if let Ok(message) = ClientMessage::read_from(&*stream_read_half) {
                            match self_clone.handle_message(
                                message,
                                &message_to_write_sender,
                                stream_ref,
                            ) {
                                Ok(return_val) => {
                                    if return_val == ProtocolReturn::DisconnectRecieved {
                                        disconnect_notifier_sender.send(()).unwrap();
                                        return Ok(());
                                    }
                                }
                                Err(err) => {
                                    stream_error_notifier_sender.send(err).unwrap();
                                    return Err(ProtocolError::StreamError);
                                }
                            }
                        }
                    });

                    let _handle_write_messages = threadpool.execute(move || {
                        //let self_ref = Arc::new(self.clone());
                        loop {
                            if disconnect_notifier_receiver.try_recv().is_ok() {
                                break;
                            }

                            //todo: will message
                            // if let Ok(err) = stream_error_notifier_receiver.try_recv() {
                            //     if err == ProtocolError::StreamError
                            //         || err == ProtocolError::AbnormalDisconnection
                            //     {
                            //         let client_id_guard = client_id;
                            //         #[allow(clippy::type_complexity)]
                            //         let clients_ids_guard = match clients_ids_clone.read() {
                            //             Ok(clients_ids_guard) => clients_ids_guard,
                            //             Err(_) => break,
                            //         };
                            //         if let Some((_, will_message)) =
                            //             clients_ids_guard.get(&*client_id_guard)
                            //         {
                            //             let will_message = match will_message {
                            //                 Some(will_message) => will_message,
                            //                 None => break,
                            //             };
                            //             // let self_clone2 = self_ref.clone();
                            //             // self_clone2
                            //             //     .clone()
                            //             //     .send_last_will(will_message, topics_clone);
                            //         }
                            //     }
                            // }

                            if let Ok(message) = message_to_write_receiver.try_recv() {
                                println!("mensaje {:?}", message);
                                match message.write_to(&*stream_write_half) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        eprintln!("{}", e);
                                        break;
                                    }
                                }
                            }
                        }
                    });
                }
                Err(_) => return Err(ProtocolError::StreamError),
            }
        }
    }

    fn process_input_command<R: BufRead>(&mut self, reader: R) -> Result<(), ProtocolError> {
        let mut iterator = reader.lines();

        if let Some(command) = iterator.next() {
            match command {
                Ok(c) => match c.trim().to_lowercase().as_str() {
                    "shutdown" => {
                        println!("Cerrando Broker");

                        match self.broker_exit() {
                            Ok(_) => {
                                println!("El Broker se ha cerrado exitosamente.");
                                exit(0);
                            }
                            Err(err) => return Err(err),
                        }
                    }
                    _ => {
                        return Err(ProtocolError::InvalidCommand(
                            "Comando no reconocido".to_string(),
                        ))
                    }
                },
                Err(e) => return Err(ProtocolError::InvalidCommand(e.to_string())),
            }
        }
        Ok(())
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
            _ => Err(ProtocolError::UnspecifiedError(
                "Error al convertir el mensaje".to_string(),
            )),
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
        if let Some((Some(stream), _)) = clients.get(&user.client_id) {
            let mut stream_clone = stream.try_clone().expect("Error al clonar el stream");
            message.write_to(&mut stream_clone).map_err(|_| true)?;
            println!("Mensaje enviado a {}", user.client_id);
            Ok(())
        } else {
            Err(true)
        }
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
            .ok_or(ProtocolError::UnspecifiedError(
                "Topic not found".to_string(),
            ))?;
        let users = topic.get_users_from_topic();

        for user in users {
            match self.send_message_to_user(&user, &mensaje) {
                Ok(_) => (),
                Err(_) => {
                    if ClientConfig::client_is_online(user.client_id.clone()) {
                        return Err(ProtocolError::UnspecifiedError(
                            "Error while sending the message: the client is online and not receiving messages".to_string(),
                        ));
                    } else {
                        let _ = ClientConfig::add_offline_message(
                            user.client_id.clone(),
                            message.clone(),
                        );
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
                    println!("Unsubscribe successfull");
                    reason_code = SUCCESS_HEX;
                }
                _ => {
                    println!("Non specified error");
                    reason_code = UNSPECIFIED_ERROR_HEX;
                }
            }
        } else {
            println!("Non specified error");
            reason_code = UNSPECIFIED_ERROR_HEX;
        }
        println!("Reason Code {:?}", reason_code);

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
        match self.handle_publish(will_publish, topics, will_topic.to_string()) {
            Ok(_) => (),
            Err(_) => println!("Error al enviar el Last Will"),
        }
    }

    /// Lee del stream un mensaje y lo procesa
    /// Devuelve un ProtocolReturn con informacion del mensaje recibido
    /// O ProtocolError en caso de error
    pub fn handle_message(
        &self,
        message: ClientMessage,
        message_to_write_sender: &Sender<BrokerMessage>,
        client_stream_ref: Arc<TcpStream>,
    ) -> Result<ProtocolReturn, ProtocolError> {
        let topics = self.topics.clone();
        let packets = self.packets.clone();
        let clients_auth_info = self.clients_auth_info.clone();

        match message {
            ClientMessage::Connect { 0: connect } => {
                println!("Connect received");

                // let cloned_stream = match stream.try_clone() {
                //     Ok(stream) => stream,
                //     Err(_) => return Err(ProtocolError::StreamError),
                // };

                let will_message = connect.clone().give_will_message();

                if ClientConfig::client_exists(connect.client_id.clone()) {
                    // si el path del client existe, eliminarlo
                    ClientConfig::delete_client_file(connect.client_id.clone())?;

                    // println!("Loading client from file");
                    // if let Err(e) =
                    //     ClientConfig::change_client_state(connect.client_id.clone(), true)
                    // {
                    //     return Err(ProtocolError::UnspecifiedError(e.to_string()));
                    // }

                    // if let Ok(mut clients) = self.clients_ids.write() {
                    //     clients.remove(&connect.client_id);
                    //     clients.insert(
                    //         connect.client_id.clone(),
                    //         (Some(cloned_stream), will_message),
                    //     );
                    // } else {
                    //     return Err(ProtocolError::WriteError);
                    // }
                }
                println!("Creating new client");
                if let Err(e) = ClientConfig::create_client_log_in_json(connect.client_id.clone()) {
                    return Err(ProtocolError::UnspecifiedError(e.to_string()));
                }
                if let Ok(mut clients) = self.clients_ids.write() {
                    clients.insert(
                        connect.client_id.clone(),
                        (Some(client_stream_ref), will_message),
                    );
                } else {
                    return Err(ProtocolError::WriteError);
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
                println!("Sending Connack");
                match message_to_write_sender.send(connack) {
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
                    match message_to_write_sender.send(puback) {
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
                println!("Subscribe received");
                let msg = ClientMessage::Subscribe {
                    packet_id,
                    properties: properties.clone(),
                    payload: payload.clone(),
                };

                Broker::save_packet(packets.clone(), msg, packet_id);

                let packet_id_bytes: [u8; 2] = packet_id.to_be_bytes();

                let reason_code = Broker::handle_subscribe(
                    topics.clone(),
                    payload.topic.clone(),
                    payload.clone(),
                )?;

                let suback = BrokerMessage::Suback {
                    packet_id_msb: packet_id_bytes[0],
                    packet_id_lsb: packet_id_bytes[1],
                    reason_code,
                };
                println!("Sending Suback");
                match message_to_write_sender.send(suback) {
                    Ok(_) => {
                        println!("Suback sent");
                        let _ = ClientConfig::add_new_subscription(
                            payload.client_id.clone(),
                            payload.topic.clone(),
                        );
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

                let reason_code = Broker::handle_unsubscribe(
                    topics.clone(),
                    payload.topic.clone(),
                    payload.clone(),
                )?;

                let unsuback = BrokerMessage::Unsuback {
                    packet_id_msb: packet_id_bytes[0],
                    packet_id_lsb: packet_id_bytes[1],
                    reason_code,
                };

                println!("Enviando un Unsuback");
                match message_to_write_sender.send(unsuback) {
                    Ok(_) => {
                        println!("Unsuback enviado");
                        let _ = ClientConfig::remove_subscription(
                            payload.client_id.clone(),
                            payload.topic.clone(),
                        );
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

                // elimino el client_id de clients_ids
                if let Ok(mut lock) = self.clients_ids.write() {
                    lock.remove(&client_id);
                } else {
                    return Err(ProtocolError::WriteError);
                }

                _ = ClientConfig::change_client_state(client_id.clone(), false);

                return Ok(ProtocolReturn::DisconnectRecieved);
            }
            ClientMessage::Pingreq => {
                println!("Recibí un Pingreq");
                let pingresp = BrokerMessage::Pingresp;
                println!("Enviando un Pingresp");
                match message_to_write_sender.send(pingresp) {
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

                        match message_to_write_sender.send(connack) {
                            Ok(_) => return Ok(ProtocolReturn::ConnackSent),
                            Err(err) => {
                                println!("{:?}", err);
                            }
                        }
                    }
                }
            }
        }
        Err(ProtocolError::UnspecifiedError(
            "Error al recibir mensaje".to_string(),
        ))
    }

    pub fn broker_exit(&self) -> Result<(), ProtocolError> {
        let clients = self
            .clients_ids
            .read()
            .map_err(|e| ProtocolError::UnspecifiedError(e.to_string()))?;
        for (client_id, (stream, _)) in clients.iter() {
            if let Some(stream) = stream {
                let disconnect = BrokerMessage::Disconnect {
                    reason_code: 0,
                    session_expiry_interval: 0,
                    reason_string: "SERVER_SHUTDOWN".to_string(),
                    user_properties: Vec::new(),
                };

                match stream.try_clone() {
                    Ok(mut cloned_stream) => {
                        match disconnect.write_to(&mut cloned_stream) {
                            Ok(_) => {
                                println!("Disconnect sent to {}", client_id);
                            }
                            Err(e) => return Err(ProtocolError::UnspecifiedError(e.to_string())),
                        }

                        match cloned_stream.shutdown(Shutdown::Both) {
                            Ok(_) => {
                                println!("Stream closed");
                            }
                            Err(e) => return Err(ProtocolError::UnspecifiedError(e.to_string())),
                        }
                    }
                    Err(e) => return Err(ProtocolError::UnspecifiedError(e.to_string())),
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
    use std::io::{Cursor, Write};
    use std::thread;

    #[test]
    fn test_01_getting_starting_topics_ok() -> Result<(), ProtocolError> {
        let file_path = "./src/monitoring/topics.txt";
        let topics = Broker::get_broker_starting_topics(file_path)?;

        let mut expected_topics = HashMap::new();
        let topic_readings = Broker::process_topic_config_file(file_path)?;

        for topic in topic_readings {
            expected_topics.insert(topic, Topic::new());
        }

        for (topic_name, _topic) in topics {
            assert!(expected_topics.contains_key(&topic_name));
        }

        Ok(())
    }

    #[test]
    fn test_02_reading_config_files_err() {
        let topics: Result<HashMap<String, Topic>, ProtocolError> =
            Broker::get_broker_starting_topics("./aca/estan/los/topics");
        let clients_auth_info = Broker::process_clients_file("./ahperoacavanlosclientesno");

        assert!(topics.is_err());
        assert!(clients_auth_info.is_err());
    }

    #[test]
    fn test_03_processing_set_of_args() -> Result<(), ProtocolError> {
        let args_ok = vec!["0.0.0.0".to_string(), "5000".to_string()];
        let args_err = vec!["este_port_abrira_tu_corazon".to_string()];

        let processing_good_args_result = Broker::process_starting_args(args_ok);
        let processing_bad_args_result = Broker::process_starting_args(args_err);

        assert!(processing_bad_args_result.is_err());
        assert!(processing_good_args_result.is_ok());

        let resulting_address = processing_good_args_result.unwrap();

        assert_eq!(resulting_address, "0.0.0.0:5000".to_string());

        Ok(())
    }

    #[test]
    fn test_04_processing_clients_auth_info_ok() -> Result<(), ProtocolError> {
        let file_path = "./src/monitoring/clients.txt";
        let clients_auth_info = Broker::process_clients_file(file_path)?;

        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => return Err(ProtocolError::ReadingClientsFileError),
        };

        let expected_clients = Broker::read_clients_file(&file)?;

        for (client_id, _auth_info) in clients_auth_info {
            assert!(expected_clients.contains_key(&client_id));
        }

        Ok(())
    }

    #[test]
    fn test_05_processing_input_commands() -> Result<(), ProtocolError> {
        let mut broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()])?;
        let good_command = b"shutdown\n";
        let cursor_one = Cursor::new(good_command);

        let bad_command = b"apagate\n";
        let cursor_two = Cursor::new(bad_command);

        assert!(broker.process_input_command(cursor_one).is_ok());
        assert!(broker.process_input_command(cursor_two).is_err());

        Ok(())
    }

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

        // Write a ClientMessage to the stream.
        // You'll need to replace this with a real ClientMessage.
        let mut result: Result<(), ProtocolError> = Err(ProtocolError::UnspecifiedError(
            "Error al leer mensaje".to_string(),
        ));
        let broker = match Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]) {
            Ok(broker) => broker,
            Err(_) => return,
        };

        // Accept the connection and pass the stream to the function.
        if let Ok((stream, _)) = listener.accept() {
            // Perform your assertions here
            result = broker.handle_client(stream);
        }

        // Check that the function returned Ok.
        // You might want to add more checks here, depending on what
        // handle_client is supposed to do.
        assert!(result.is_err());
    }
}
