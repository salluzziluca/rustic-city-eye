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
    reason_code::{
        NO_MATCHING_SUBSCRIBERS_HEX, SUB_ID_DUP_HEX, SUCCESS_HEX, UNSPECIFIED_ERROR_HEX,
    },
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

    subs: Vec<u32>,

    clients_ids: Arc<Vec<String>>,
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
            subs: Vec::new(),
            clients_ids: Arc::new(Vec::new()),
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
                    let subs_clone = self.subs.clone();
                    let clients_ids_clone = self.clients_ids.clone();

                    threadpool.execute(move || {
                        // Use the cloned reference
                        let _ = Broker::handle_client(
                            stream,
                            topics_clone,
                            packets_clone,
                            subs_clone,
                            clients_ids_clone,
                        );
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
        clients_ids: Arc<Vec<String>>,
    ) -> Result<(), ProtocolError> {
        loop {
            let cloned_stream = match stream.try_clone() {
                Ok(stream) => stream,
                Err(_) => return Err(ProtocolError::StreamError),
            };
            match handle_messages(
                cloned_stream,
                topics.clone(),
                packets.clone(),
                _subs.clone(),
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
        let mut lock = packets.write().unwrap();

        lock.insert(packet_id, message);
    }
}

/// Lee del stream un mensaje y lo procesa
/// Devuelve un ProtocolReturn con informacion del mensaje recibido
/// O ProtocolError en caso de error
pub fn handle_messages(
    mut stream: TcpStream,
    topics: HashMap<String, Topic>,
    packets: Arc<RwLock<HashMap<u16, ClientMessage>>>,
    _subs: Vec<u32>,
    mut clients_ids: Arc<Vec<String>>,
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

            if clients_ids.contains(&client_id) {
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
            Arc::make_mut(&mut clients_ids).push(client_id);
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

            let reason_code = Broker::handle_publish(msg, topics.clone(), topic_name)?;

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
        let subs = vec![];
        let clients_ids = Arc::new(vec![]);

        // Write a ClientMessage to the stream.
        // You'll need to replace this with a real ClientMessage.
        let mut result: Result<(), ProtocolError> = Err(ProtocolError::UnspecifiedError);

        // Accept the connection and pass the stream to the function.
        if let Ok((stream, _)) = listener.accept() {
            // Perform your assertions here
            result = Broker::handle_client(stream, topics, packets, subs, clients_ids);
        }

        // Check that the function returned Ok.
        // You might want to add more checks here, depending on what
        // handle_client is supposed to do.
        assert!(result.is_err());
    }
}
