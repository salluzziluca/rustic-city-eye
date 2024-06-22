use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock},
};

use crate::mqtt::{
    broker_config::BrokerConfig,
    broker_message::BrokerMessage,
    client_message::ClientMessage,
    connack_properties::ConnackProperties,
    protocol_error::ProtocolError,
    protocol_return::ProtocolReturn,
    reason_code::{SUB_ID_DUP_HEX, SUCCESS_HEX, UNSPECIFIED_ERROR_HEX},
    subscription::Subscription,
    topic::Topic,
};

use crate::utils::threadpool::ThreadPool;

static SERVER_ARGS: usize = 2;

const THREADPOOL_SIZE: usize = 20;

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
    clients_ids: Arc<RwLock<HashMap<String, TcpStream>>>,

    /// Contiene los clientes desconectados del broker y sus mensajes pendientes.
    offline_clients: Arc<RwLock<HashMap<String, Vec<ClientMessage>>>>,
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

        let broker_config = BrokerConfig::new(address)?;

        let (address, topics) = broker_config.get_broker_config();

        let packets = HashMap::new();

        Ok(Broker {
            address,
            topics,
            packets: Arc::new(RwLock::new(packets)),
            clients_ids: Arc::new(RwLock::new(HashMap::new())),
            offline_clients: Arc::new(RwLock::new(HashMap::new())),
        })
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
                    let clients_ids_clone = self.clients_ids.clone();
                    let self_ref = Arc::new(self.clone()); // wrap `self` in an Arc

                    threadpool.execute(move || {
                        // Use the cloned reference
                        let _ = <Broker as Clone>::clone(&self_ref).handle_client(
                            stream,
                            topics_clone,
                            packets_clone,
                            clients_ids_clone,
                        );
                    });
                }
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }

    /// Se encarga del manejo de los mensajes del cliente. Envia los ACKs correspondientes.
    pub fn handle_client(
        self,
        stream: TcpStream,
        topics: HashMap<String, Topic>,
        packets: Arc<RwLock<HashMap<u16, ClientMessage>>>,
        clients_ids: Arc<RwLock<HashMap<String, TcpStream>>>,
    ) -> Result<(), ProtocolError> {
        loop {
            let cloned_stream = match stream.try_clone() {
                Ok(stream) => stream,
                Err(_) => return Err(ProtocolError::StreamError),
            };
            match self.handle_messages(
                cloned_stream,
                topics.clone(),
                packets.clone(),
                clients_ids.clone(),
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
            match topic.add_user_to_topic(subscription) {
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

    fn handle_publish(
        &self,
        message: ClientMessage,
        mut topics: HashMap<String, Topic>,
        topic_name: String,
    ) -> Result<u8, ProtocolError> {
        // verifico si el topic exite
        if let Some(topic) = topics.get_mut(&topic_name) {
            //obtengo los users que corresponden a ese topic
            let users = topic.get_topic_users();
            for user in users {
                //verifico si el user esta conectado
                let mut es_qos_1 = false;
                let mut esta_offline = false;
                let m = message.clone();
                if let Some(mut stream) = self.clients_ids.read().unwrap().get(&user.client_id) {
                    //envio el mensaje al user

                    match m.write_to(&mut stream) {
                        Ok(_) => {
                            println!("Mensaje enviado a {}", user.client_id);
                        }
                        Err(_) => {
                            // si es qos 1 me guardo el mensjae
                            if user.qos == 1 {
                                es_qos_1 = true;
                            }
                        }
                    }
                    // si el mensaje es qos 1, envio el ack
                } else if let Some(_stream) =
                    self.offline_clients.read().unwrap().get(&user.client_id)
                {
                    //guardo el mensaje para enviarlo cuando el user se conecte
                    esta_offline = true;
                }

                if esta_offline && es_qos_1 {
                    let mut lock = self.offline_clients.write().unwrap();
                    if let Some(messages) = lock.get_mut(&user.client_id) {
                        messages.push(m);
                    } else {
                        lock.insert(user.client_id, vec![m]);
                    }
                }
            }
        }

        Ok(0x80_u8) //Unspecified Error reason code
    }

    /// Maneja la desubscripcion de un cliente a un topic
    /// Devuelve el reason code correspondiente a si la desubscripcion fue exitosa o no
    /// Si el reason code es 0, el cliente se ha desuscrito exitosamente.
    fn handle_unsubscribe(
        mut topics: HashMap<String, Topic>,
        topic_name: String,
        usersubscription: Subscription,
    ) -> Result<u8, ProtocolError> {
        let reason_code;

        if let Some(topic) = topics.get_mut(&topic_name) {
            match topic.remove_user_from_topic(usersubscription) {
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
        let mut lock = packets.write().unwrap();

        lock.insert(packet_id, message);
    }

    /// Lee del stream un mensaje y lo procesa
    /// Devuelve un ProtocolReturn con informacion del mensaje recibido
    /// O ProtocolError en caso de error
    pub fn handle_messages(
        &self,
        mut stream: TcpStream,
        topics: HashMap<String, Topic>,
        packets: Arc<RwLock<HashMap<u16, ClientMessage>>>,
        clients_ids: Arc<RwLock<HashMap<String, TcpStream>>>,
    ) -> Result<ProtocolReturn, ProtocolError> {
        let mensaje = match ClientMessage::read_from(&mut stream) {
            Ok(mensaje) => mensaje,
            Err(_) => return Err(ProtocolError::StreamError),
        };
        match mensaje {
            ClientMessage::Connect {
                clean_start: _,
                last_will_flag: _,
                last_will_qos: _,
                last_will_retain: _,
                username: _,
                password: _,
                keep_alive: _,
                properties: _,
                client_id,
                will_properties: _,
                last_will_topic: _,
                last_will_message: _,
            } => {
                println!("Recibí un Connect");

                if clients_ids.read().unwrap().contains_key(&client_id) {
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

                if self
                    .offline_clients
                    .read()
                    .unwrap()
                    .contains_key(&client_id)
                {
                    let offline_clients = self.offline_clients.read().unwrap();
                    let pending_messages = offline_clients.get(&client_id).unwrap();
                    for message in pending_messages {
                        match message.write_to(&mut stream) {
                            Ok(_) => {
                                println!("Mensaje enviado a {}", client_id);
                            }
                            Err(err) => {
                                println!("Error al enviar mensaje: {:?}", err);
                            }
                        }
                    }
                }

                //si está en offline_clients lo elimino de ahí
                let mut lock = self.offline_clients.write().unwrap();
                lock.remove(&client_id);

                //clona stream con ok err
                let cloned_stream = match stream.try_clone() {
                    Ok(stream) => stream,
                    Err(_) => return Err(ProtocolError::StreamError),
                };

                clients_ids
                    .write()
                    .unwrap()
                    .insert(client_id.clone(), cloned_stream);

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

                let mut reason_code_vec = Vec::new();

                for p in payload {
                    let reason_code =
                        Broker::handle_subscribe(topics.clone(), p.topic.clone(), p.clone())?;

                    reason_code_vec.push(reason_code);
                }

                if reason_code_vec.iter().any(|&x| x != SUCCESS_HEX) {
                    let suback = BrokerMessage::Suback {
                        packet_id_msb: packet_id_bytes[0],
                        packet_id_lsb: packet_id_bytes[1],
                        reason_code: UNSPECIFIED_ERROR_HEX,
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

                let mut reason_code_vec = Vec::new();

                for p in payload {
                    let reason_code =
                        Broker::handle_unsubscribe(topics.clone(), p.topic.clone(), p.clone())?;
                    reason_code_vec.push(reason_code);
                }

                if reason_code_vec.iter().any(|&x| x != SUCCESS_HEX) {
                    let suback = BrokerMessage::Suback {
                        packet_id_msb: packet_id_bytes[0],
                        packet_id_lsb: packet_id_bytes[1],
                        reason_code: UNSPECIFIED_ERROR_HEX,
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

                let unsuback = BrokerMessage::Unsuback {
                    packet_id_msb: packet_id_bytes[0],
                    packet_id_lsb: packet_id_bytes[1],
                    reason_code: SUCCESS_HEX,
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
                client_id: _,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::{Arc, RwLock};
    use std::thread;

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

        let clients_ids = Arc::new(RwLock::new(HashMap::new()));

        // Write a ClientMessage to the stream.
        // You'll need to replace this with a real ClientMessage.
        let mut result: Result<(), ProtocolError> = Err(ProtocolError::UnspecifiedError);
        let broker = Broker::new(vec!["127.0.0.1".to_string(), "5000".to_string()]).unwrap();
        // Accept the connection and pass the stream to the function.
        if let Ok((stream, _)) = listener.accept() {
            // Perform your assertions here
            result = broker.handle_client(stream, topics, packets, clients_ids);
        }

        // Check that the function returned Ok.
        // You might want to add more checks here, depending on what
        // handle_client is supposed to do.
        assert!(result.is_err());
    }
}
