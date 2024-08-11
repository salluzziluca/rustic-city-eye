use rand::Rng;
use rustls::{ClientConfig, ClientConnection, KeyLogFile, RootCertStore, StreamOwned};
use std::{
    fs::File,
    io::BufReader,
    net::{Shutdown, TcpStream},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use crate::{
    mqtt::{
        broker_message::BrokerMessage, client_message::ClientMessage,
        messages_config::MessagesConfig, protocol_error::ProtocolError,
    },
    utils::threadpool::ThreadPool,
};

use super::{client_message, client_return::ClientReturn};

pub trait ClientTrait {
    fn client_run(&mut self) -> Result<(), ProtocolError>;
    fn clone_box(&self) -> Box<dyn ClientTrait>;
    fn assign_packet_id(&self) -> u16;
    fn get_publish_end_channel(
        &self,
    ) -> Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Box<(dyn MessagesConfig + Send + 'static)>>>>;
    fn get_client_id(&self) -> String;
    fn disconnect_client(&self) -> Result<(), ProtocolError>;
}
impl Clone for Box<dyn ClientTrait> {
    fn clone(&self) -> Box<dyn ClientTrait> {
        self.clone_box()
    }
}
#[derive(Debug, Clone)]
pub struct Client {
    receiver_channel: Arc<Mutex<Receiver<Box<dyn MessagesConfig + Send>>>>,

    // stream es el socket que se conecta al broker
    stream: Arc<StreamOwned<ClientConnection, TcpStream>>,

    // client_id es el identificador del cliente
    pub client_id: String,

    packets_ids: Arc<Vec<u16>>,

    sender_channel: Option<Sender<ClientMessage>>,
}

impl Client {
    /// Se intenta connectar al servidor corriendo en address.
    ///
    /// Si la conexion es exitosa, inmediatamente envia un packet del tipo Connect(que
    /// ingresa como parametro).
    ///
    /// Si el enviado del Connect es exitoso, se espera una respuesta del Broker, la cual
    /// debe ser un Connack packet.
    ///
    /// Al recibir un Connack, se ve su reason_code, y si este es 0x00(conexion exitosa), se crea la instancia
    /// del Client.
    pub fn new(
        receiver_channel: Receiver<Box<dyn MessagesConfig + Send>>,
        address: String,
        connect: client_message::Connect,
        sender_channel: Sender<ClientMessage>,
    ) -> Result<Client, ProtocolError> {
        let stream = match TcpStream::connect(address) {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::ConectionError),
        };

        let client_id = connect.get_client_id().to_string();
        let connect_message = ClientMessage::Connect(connect);

        let tls_stream = Client::build_tls_stream(stream)?;

        println!("Sending Connect message to Broker");

        match connect_message.write_to(tls_stream.get_ref()) {
            Ok(()) => println!("Connect message send"),
            Err(e) => return Err(e),
        }

        if let Ok(message) = BrokerMessage::read_from(tls_stream.get_ref()) {
            match message {
                BrokerMessage::Connack {
                    session_present: _,
                    reason_code,
                    properties: _,
                } => {
                    println!("Connack received");
                    match reason_code {
                        0x00_u8 => {
                            println!("Successful connection!");

                            Ok(Client {
                                receiver_channel: Arc::new(Mutex::new(receiver_channel)),
                                stream: tls_stream,
                                packets_ids: Arc::new(Vec::new()),
                                sender_channel: Some(sender_channel),
                                client_id,
                            })
                        }
                        _ => {
                            println!("Authentication failed: reason code {}", reason_code);
                            Err(ProtocolError::AuthError)
                        }
                    }
                }
                _ => Err(ProtocolError::ExpectedConnack),
            }
        } else {
            Err(ProtocolError::NotReceivedMessageError)
        }
    }

    fn build_tls_stream(stream: TcpStream) -> Result<Arc<StreamOwned<ClientConnection, TcpStream>>, ProtocolError> {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let mut root_store = RootCertStore::empty();
        let mut cert_file = Client::open_file("./src/mqtt/certs/cert.pem")?;
        root_store.add_parsable_certificates(
            rustls_pemfile::certs(&mut cert_file).map(|result| result.unwrap()),
        );

        let mut config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        config.key_log = Arc::new(KeyLogFile::new());

        let server_name = "rustic_city_eye".try_into().unwrap();
        match ClientConnection::new(Arc::new(config), server_name) {
            Ok(c) => Ok(Arc::new(StreamOwned::new(c, stream))),
            Err(e) => Err(ProtocolError::ClientConnectionError(e.to_string())),
        }
    }

    fn open_file(file_path: &str) -> Result<BufReader<File>, ProtocolError> {
        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(e) => return Err(ProtocolError::OpenFileError(e.to_string())),
        };

        Ok(BufReader::new(file))
    }

    /// Recibe un string que indica la razón de la desconexión y un stream y envia un disconnect message al broker
    /// segun el str reason recibido, modifica el reason_code y el reason_string del mensaje
    ///
    /// devuelve el packet_id del mensaje enviado o un ClientError en caso de error
    pub fn handle_disconnect(
        client_id: String,
        reason: &str,
        session_expiry_interval: u32,
    ) -> ClientMessage {
        let reason_code: u8;
        let reason_string: String;
        match reason {
            "normal" => {
                reason_code = 0x00;
                reason_string =
                    "Close the connection normally. Do not send the Will Message.".to_string();
            }
            "with_will" => {
                reason_code = 0x04;
                reason_string = "The Client wishes to disconnect but requires that the Server also publishes its Will Message"
                    .to_string();
            }
            _ => {
                reason_code = 0x80;
                reason_string = "The Connection is closed but the sender either does not wish to reveal reason or none of the other Reason Codes apply. "
                    .to_string();
            }
        }

        ClientMessage::Disconnect {
            reason_code,
            session_expiry_interval,
            reason_string,
            client_id,
        }
    }

    /// Se encarga de que el cliente este funcionando correctamente.
    /// El Client debe encargarse de dos tareas: leer mensajes que le lleguen del Broker.
    /// Estos mensajes pueden ser tanto acks (Connack, Puback, etc.)
    /// como pubs que vengan por parte otros clientes (mediante el broker)
    /// a cuyos topics esté subscrito.
    ///
    /// Su segunda tarea es enviar mensajes:
    /// puede enviar mensajes como Publish, Suscribe, etc.
    ///
    /// Las dos tareas del Client se deben ejecutar concurrentemente,
    /// por eso su tilizan dos threads(uno de lectura y otro de escritura),
    /// ambos threads deben compaten el recurso TcpStream.
    ///
    /// El thread de escritura (write_messages) recibe por consola los mensajes a enviar.
    /// Si logra enviar los mensajes correctamente, envia el pacjet id mediante el channel
    ///
    /// El thread de lectura (read_messages) se encarga de leer los mensajes que le llegan del broker.
    pub fn client_run(&mut self) -> Result<(), ProtocolError> {
        let threadpool = ThreadPool::new(5);

        let stream_write_half = Arc::clone(&self.stream);
        let stream_read_half = Arc::clone(&self.stream);
        let stream_ref = Arc::clone(&self.stream);

        let (pending_id_messages_sender, pending_id_messages_receiver) = mpsc::channel();
        let (puback_notify_sender, puback_notify_receiver) = mpsc::channel();

        let receiver_channel = self.receiver_channel.clone();

        let (disconnect_sender, disconnect_receiver) = mpsc::channel();
        let client_id_clone = self.client_id.clone();
        let packet_ids_ref = Arc::clone(&self.packets_ids);

        let _write_messages = threadpool.execute(move || {
            Client::write_messages(
                stream_write_half,
                receiver_channel,
                pending_id_messages_sender,
                puback_notify_receiver,
                disconnect_receiver,
                packet_ids_ref,
            )
        });

        let sender_channel_clone = self.sender_channel.clone();
        if let Some(sender_channel) = sender_channel_clone {
            let _read_messages = threadpool.execute(move || {
                match Client::receive_messages(
                    stream_read_half,
                    pending_id_messages_receiver,
                    puback_notify_sender,
                    sender_channel,
                    disconnect_sender,
                    client_id_clone,
                ) {
                    Ok(_) => {
                        stream_ref.get_ref().shutdown(Shutdown::Both).map_err(|_| {
                            ProtocolError::ShutdownError("Error al cerrar el stream".to_string())
                        })?;
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            });
        }

        Ok(())
    }

    pub fn receive_messages(
        stream: Arc<StreamOwned<ClientConnection, TcpStream>>,
        pending_id_messages_receiver: Receiver<u16>,
        puback_notify_sender: Sender<bool>,
        sender_channel: Sender<ClientMessage>,
        disconnect_sender: Sender<bool>,
        client_id: String,
    ) -> Result<(), ProtocolError> {
        let mut pending_messages = Vec::new();

        loop {
            if let Ok(packet) = pending_id_messages_receiver.try_recv() {
                if !pending_messages.contains(&packet) {
                    pending_messages.push(packet);
                }
            }

            if let Ok(message) = BrokerMessage::read_from(stream.get_ref()) {
                match Client::handle_message(
                    message,
                    pending_messages.clone(),
                    puback_notify_sender.clone(),
                    sender_channel.clone(),
                    client_id.clone(),
                ) {
                    Ok(return_value) => {
                        if return_value == ClientReturn::DisconnectRecieved {
                            disconnect_sender.send(true).expect("Error al desconectar");
                            return Ok(());
                        }
                    }
                    Err(err) => return Err(err),
                }
            }
        }
    }

    /// Lee del stream un mensaje y lo procesa
    /// Devuelve un ClientReturn con informacion del mensaje recibido
    /// O ProtocolError en caso de error
    pub fn handle_message(
        message: BrokerMessage,
        pending_messages: Vec<u16>,
        puback_notify_sender: Sender<bool>,
        sender_channel: Sender<ClientMessage>,
        client_id: String,
    ) -> Result<ClientReturn, ProtocolError> {
        match message {
            BrokerMessage::Connack {
                session_present: _,
                reason_code: _,
                properties: _,
            } => {
                println!("Connack received");
                Ok(ClientReturn::ConnackReceived)
            }
            BrokerMessage::Puback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
            } => handle_puback(
                &pending_messages,
                packet_id_msb,
                packet_id_lsb,
                puback_notify_sender,
            ),
            BrokerMessage::Disconnect {
                reason_code,
                session_expiry_interval,
                reason_string,
                user_properties: _,
            } => {
                handle_disconnect(
                    reason_string,
                    &sender_channel,
                    reason_code,
                    session_expiry_interval,
                    client_id,
                );

                Ok(ClientReturn::DisconnectRecieved)
            }
            BrokerMessage::Suback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
            } => {
                handle_suback(&pending_messages, packet_id_msb, packet_id_lsb);

                Ok(ClientReturn::SubackRecieved)
            }
            BrokerMessage::PublishDelivery {
                packet_id,
                topic_name,
                qos,
                retain_flag,
                dup_flag,
                properties,
                payload,
            } => {
                handle_publish_delivery(
                    sender_channel,
                    packet_id,
                    topic_name,
                    qos,
                    retain_flag,
                    dup_flag,
                    properties,
                    payload,
                );

                Ok(ClientReturn::PublishDeliveryRecieved)
            }
            BrokerMessage::Unsuback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
            } => {
                handle_unsuback(pending_messages, packet_id_msb, packet_id_lsb);
                Ok(ClientReturn::UnsubackRecieved)
            }
            BrokerMessage::Pingresp => Ok(ClientReturn::PingrespRecieved),
            BrokerMessage::Auth {
                reason_code: _,
                authentication_method: _,
                authentication_data: _,
                reason_string: _,
                user_properties: _,
            } => Ok(ClientReturn::AuthRecieved),
        }
    }

    pub fn disconnect_client(&mut self) -> Result<(), ProtocolError> {
        self.sender_channel = None;

        match self.stream.get_ref().shutdown(Shutdown::Both) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Client: Error while shutting down stream: {:?}", e);
                Err(ProtocolError::ShutdownError(e.to_string()))
            }
        }
    }

    /// Esta funcion se encarga de la escritura de mensajes que recibe mediante el channel.
    ///
    ///
    /// Si el mensaje es un publish con qos 1, se envia el mensaje y se espera un puback. Si no se recibe, espera 0.5 segundos y reenvia el mensaje. aumentando en 1 el dup_flag, indicando que es al vez numero n que se envia el publish.
    #[allow(clippy::too_many_arguments)]
    fn write_messages(
        stream: Arc<StreamOwned<ClientConnection, TcpStream>>,
        receiver_channel: Arc<Mutex<Receiver<Box<dyn MessagesConfig + Send>>>>,
        pending_id_messages_sender: Sender<u16>,
        puback_notify_receiver: Receiver<bool>,
        disconnect_receiver: Receiver<bool>,
        packet_ids: Arc<Vec<u16>>,
    ) -> Result<(), ProtocolError> {
        loop {
            if let Ok(disconnect_status) = disconnect_receiver.try_recv() {
                if disconnect_status {
                    return Ok(());
                }
            }

            let lock = match receiver_channel.lock() {
                Ok(lock) => lock,
                Err(_) => return Err(ProtocolError::StreamError),
            };
            if let Ok(message_config) = lock.recv() {
                let packet_id = Client::get_packet_id(packet_ids.to_vec());
                let message = message_config.parse_message(packet_id);

                match message {
                    ClientMessage::Publish {
                        packet_id,
                        topic_name,
                        qos,
                        retain_flag,
                        payload,
                        dup_flag,
                        properties,
                    } => {
                        if let Some(value) = write_publish(
                            packet_id,
                            topic_name,
                            qos,
                            retain_flag,
                            payload,
                            dup_flag,
                            properties,
                            &stream,
                            &pending_id_messages_sender,
                            &puback_notify_receiver,
                        ) {
                            return value;
                        }
                    }
                    ClientMessage::Disconnect {
                        reason_code: _,
                        session_expiry_interval,
                        reason_string,
                        client_id,
                    } => {
                        let disconnect = Client::handle_disconnect(
                            client_id,
                            &reason_string,
                            session_expiry_interval,
                        );

                        match disconnect.write_to(stream.get_ref()) {
                            Ok(_) => match pending_id_messages_sender.send(packet_id) {
                                Ok(_) => {}
                                Err(e) => {
                                    return Err(ProtocolError::SendError(e.to_string()));
                                }
                            },
                            Err(e) => {
                                return Err(ProtocolError::SendError(e.to_string()));
                            }
                        }
                    }
                    _ => match message.write_to(stream.get_ref()) {
                        Ok(_) => match pending_id_messages_sender.send(packet_id) {
                            Ok(_) => {}
                            Err(e) => {
                                return Err(ProtocolError::SendError(e.to_string()));
                            }
                        },
                        Err(e) => {
                            return Err(ProtocolError::SendError(e.to_string()));
                        }
                    },
                }
            }
        }
    }

    pub fn get_publish_end_channel(
        &self,
    ) -> Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Box<(dyn MessagesConfig + Send + 'static)>>>>
    {
        self.receiver_channel.clone()
    }

    pub fn get_packet_id(mut packet_ids: Vec<u16>) -> u16 {
        let mut rng = rand::thread_rng();

        let mut packet_id: u16;
        loop {
            packet_id = rng.gen();
            if packet_id != 0 && !packet_ids.contains(&packet_id) {
                packet_ids.push(packet_id);
                break;
            }
        }
        packet_id
    }
}
/// Recibe todos los campos necesarios para la escritura por stream de un mensaje Publish.
/// En caso de que este tenga una QoS == 1 y no se reiba un Puback, se reenvia el mensaje hasta recibirlo
fn write_publish(
    packet_id: u16,
    topic_name: String,
    qos: usize,
    retain_flag: usize,
    payload: crate::utils::payload_types::PayloadTypes,
    dup_flag: usize,
    properties: super::publish::publish_properties::PublishProperties,
    stream: &Arc<StreamOwned<ClientConnection, TcpStream>>,
    pending_id_messages_sender: &Sender<u16>,
    puback_notify_receiver: &Receiver<bool>,
) -> Option<Result<(), ProtocolError>> {
    let publish = ClientMessage::Publish {
        packet_id,
        topic_name: topic_name.clone(),
        qos,
        retain_flag,
        payload: payload.clone(),
        dup_flag,
        properties: properties.clone(),
    };
    match publish.write_to(stream.get_ref()) {
        Ok(_) => match pending_id_messages_sender.send(packet_id) {
            Ok(_) => {
                if qos == 1 {
                    loop {
                        thread::sleep(Duration::from_millis(20));

                        if let Ok(puback) = puback_notify_receiver.try_recv() {
                            if !puback {
                                let publish = ClientMessage::Publish {
                                    packet_id,
                                    topic_name: topic_name.clone(),
                                    qos,
                                    retain_flag,
                                    payload: payload.clone(),
                                    dup_flag: dup_flag + 1,
                                    properties: properties.clone(),
                                };

                                match publish.write_to(stream.get_ref()) {
                                    Ok(_) => match pending_id_messages_sender.send(packet_id) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            return Some(Err(ProtocolError::SendError(
                                                e.to_string(),
                                            )))
                                        }
                                    },
                                    Err(e) => {
                                        return Some(Err(ProtocolError::SendError(e.to_string())))
                                    }
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                return Some(Err(ProtocolError::SendError(e.to_string())));
            }
        },
        Err(e) => {
            return Some(Err(ProtocolError::SendError(e.to_string())));
        }
    }
    None
}

fn handle_unsuback(pending_messages: Vec<u16>, packet_id_msb: u8, packet_id_lsb: u8) {
    for pending_message in &pending_messages {
        let packet_id_bytes: [u8; 2] = pending_message.to_be_bytes();
        if packet_id_bytes[0] == packet_id_msb && packet_id_bytes[1] == packet_id_lsb {
            println!(
                "Unsuback con id {} {} recibido",
                packet_id_msb, packet_id_lsb
            );
        }
    }
}

fn handle_publish_delivery(
    sender_channel: Sender<ClientMessage>,
    packet_id: u16,
    topic_name: String,
    qos: usize,
    retain_flag: usize,
    dup_flag: usize,
    properties: super::publish::publish_properties::PublishProperties,
    payload: crate::utils::payload_types::PayloadTypes,
) {
    match sender_channel.send(ClientMessage::Publish {
        packet_id,
        topic_name,
        qos,
        retain_flag,
        dup_flag,
        properties,
        payload,
    }) {
        Ok(_) => {}
        Err(e) => println!("Error al enviar publish al sistema: {:?}", e),
    }
}

fn handle_suback(pending_messages: &Vec<u16>, packet_id_msb: u8, packet_id_lsb: u8) {
    for pending_message in pending_messages {
        let packet_id_bytes: [u8; 2] = pending_message.to_be_bytes();

        if packet_id_bytes[0] == packet_id_msb && packet_id_bytes[1] == packet_id_lsb {
            println!("suback con id {} {} recibido", packet_id_msb, packet_id_lsb);
        }
    }
}

fn handle_puback(
    pending_messages: &Vec<u16>,
    packet_id_msb: u8,
    packet_id_lsb: u8,
    puback_notify_sender: Sender<bool>,
) -> Result<ClientReturn, ProtocolError> {
    for pending_message in pending_messages {
        let packet_id_bytes: [u8; 2] = pending_message.to_be_bytes();
        if packet_id_bytes[0] == packet_id_msb && packet_id_bytes[1] == packet_id_lsb {}
    }
    match puback_notify_sender.send(true) {
        Ok(_) => Ok(ClientReturn::PubackRecieved),
        Err(e) => Err(ProtocolError::ChanellError(e.to_string())),
    }
}

fn handle_disconnect(
    reason_string: String,
    sender_channel: &Sender<ClientMessage>,
    reason_code: u8,
    session_expiry_interval: u32,
    client_id: String,
) {
    println!(
        "Recibí un Disconnect, razon de desconexión: {:?}",
        reason_string
    );

    match sender_channel.send(ClientMessage::Disconnect {
        reason_code,
        session_expiry_interval,
        reason_string,
        client_id,
    }) {
        Ok(_) => {}
        Err(e) => println!("Error al enviar Disconnect al sistema: {:?}", e),
    }
}

impl ClientTrait for Client {
    fn client_run(&mut self) -> Result<(), ProtocolError> {
        self.client_run()
    }

    fn clone_box(&self) -> Box<dyn ClientTrait> {
        Box::new(self.clone())
    }
    fn assign_packet_id(&self) -> u16 {
        Client::get_packet_id(self.packets_ids.to_vec())
    }

    fn get_publish_end_channel(
        &self,
    ) -> Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Box<(dyn MessagesConfig + Send + 'static)>>>>
    {
        self.get_publish_end_channel()
    }
    fn get_client_id(&self) -> String {
        self.client_id.clone()
    }

    fn disconnect_client(&self) -> Result<(), ProtocolError> {
        match self.stream.get_ref().shutdown(Shutdown::Both) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Client: Error while shutting down stream: {:?}", e);
                Err(ProtocolError::ShutdownError(e.to_string()))
            }
        }
    }
}

/// Lee del stream un mensaje y lo procesa
/// Devuelve un ClientReturn con informacion del mensaje recibido
/// O ProtocolError en caso de error
pub fn handle_message(
    stream: Arc<TcpStream>,
    pending_messages: Vec<u16>,
    sender: Sender<bool>,
    sender_chanell: Sender<ClientMessage>,
    client_id: String,
) -> Result<ClientReturn, ProtocolError> {
    if let Ok(message) = BrokerMessage::read_from(&*stream) {
        match message {
            BrokerMessage::Connack {
                session_present: _,
                reason_code: _,
                properties: _,
            } => {
                println!("Recibí un Connack");
                Ok(ClientReturn::ConnackReceived)
            }
            BrokerMessage::Puback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
            } => {
                for pending_message in &pending_messages {
                    let packet_id_bytes: [u8; 2] = pending_message.to_be_bytes();
                    if packet_id_bytes[0] == packet_id_msb && packet_id_bytes[1] == packet_id_lsb {}
                }
                match sender.send(true) {
                    Ok(_) => Ok(ClientReturn::PubackRecieved),
                    Err(e) => Err(ProtocolError::ChanellError(e.to_string())),
                }
            }
            BrokerMessage::Disconnect {
                reason_code,
                session_expiry_interval,
                reason_string,
                user_properties: _,
            } => {
                println!(
                    "Recibí un Disconnect, razon de desconexión: {:?}",
                    reason_string
                );

                match sender_chanell.send(ClientMessage::Disconnect {
                    reason_code,
                    session_expiry_interval,
                    reason_string,
                    client_id,
                }) {
                    Ok(_) => {}
                    Err(e) => println!("Error al enviar Disconnect al sistema: {:?}", e),
                }

                Ok(ClientReturn::DisconnectRecieved)
            }
            BrokerMessage::Suback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
            } => {
                for pending_message in &pending_messages {
                    let packet_id_bytes: [u8; 2] = pending_message.to_be_bytes();

                    if packet_id_bytes[0] == packet_id_msb && packet_id_bytes[1] == packet_id_lsb {
                        println!("suback con id {} {} recibido", packet_id_msb, packet_id_lsb);
                    }
                }

                Ok(ClientReturn::SubackRecieved)
            }
            BrokerMessage::PublishDelivery {
                packet_id,
                topic_name,
                qos,
                retain_flag,
                dup_flag,
                properties,
                payload,
            } => {
                match sender_chanell.send(ClientMessage::Publish {
                    packet_id,
                    topic_name,
                    qos,
                    retain_flag,
                    dup_flag,
                    properties,
                    payload,
                }) {
                    Ok(_) => {}
                    Err(e) => println!("Error al enviar publish al sistema: {:?}", e),
                }

                Ok(ClientReturn::PublishDeliveryRecieved)
            }
            BrokerMessage::Unsuback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
            } => {
                for pending_message in &pending_messages {
                    let packet_id_bytes: [u8; 2] = pending_message.to_be_bytes();
                    if packet_id_bytes[0] == packet_id_msb && packet_id_bytes[1] == packet_id_lsb {
                        println!(
                            "Unsuback con id {} {} recibido",
                            packet_id_msb, packet_id_lsb
                        );
                    }
                }

                println!("Recibi un mensaje {:?}", message);
                Ok(ClientReturn::UnsubackRecieved)
            }
            BrokerMessage::Pingresp => {
                println!("Recibi un mensaje {:?}", message);
                Ok(ClientReturn::PingrespRecieved)
            }
            BrokerMessage::Auth {
                reason_code: _,
                authentication_method: _,
                authentication_data: _,
                reason_string: _,
                user_properties: _,
            } => {
                println!("recibi un auth!");
                Ok(ClientReturn::AuthRecieved)
            }
        }
    } else {
        Err(ProtocolError::StreamError)
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Condvar;

    use crate::mqtt::broker::Broker;

    use super::*;

    #[test]
    fn test_assign_packet_id() {
        let args = vec!["127.0.0.1".to_string(), "5047".to_string()];
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => panic!("Error creating broker: {:?}", e),
        };

        let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
        let server_ready_clone = server_ready.clone();
        thread::spawn(move || {
            {
                let (lock, cvar) = &*server_ready_clone;
                let mut ready = lock.lock().unwrap();
                *ready = true;
                cvar.notify_all();
            }
            broker.server_run().unwrap();
        });

        // Wait for the server to start
        {
            let (lock, cvar) = &*server_ready;
            let mut ready = lock.lock().unwrap();
            while !*ready {
                ready = cvar.wait(ready).unwrap();
            }
        }
        let handle = thread::spawn(move || {
            let connect_config =
                client_message::Connect::read_connect_config("src/drones/connect_config.json")
                    .unwrap();
            let (_, rx) = mpsc::channel();
            let (tx2, _) = mpsc::channel();
            let address = "127.0.0.1:5047";
            let client = Client::new(rx, address.to_string(), connect_config, tx2).unwrap();
            let packet_id = client.assign_packet_id();
            assert_ne!(packet_id, 0);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_si_el_id_de_paquete_es_unico() {
        let args = vec!["127.0.0.1".to_string(), "5039".to_string()];
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => panic!("Error creating broker: {:?}", e),
        };

        let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
        let server_ready_clone = server_ready.clone();
        thread::spawn(move || {
            {
                let (lock, cvar) = &*server_ready_clone;
                let mut ready = lock.lock().unwrap();
                *ready = true;
                cvar.notify_all();
            }
            let _ = broker.server_run();
        });

        // Wait for the server to start
        {
            let (lock, cvar) = &*server_ready;
            let mut ready = lock.lock().unwrap();
            while !*ready {
                ready = cvar.wait(ready).unwrap();
            }
        }
        let handle = thread::spawn(move || {
            let connect_config =
                client_message::Connect::read_connect_config("src/drones/connect_config.json")
                    .unwrap();
            let (_, rx) = mpsc::channel();
            let (tx2, _) = mpsc::channel();
            let address = "127.0.0.1:5039";
            let client = Client::new(rx, address.to_string(), connect_config, tx2).unwrap();
            let packet_id = client.assign_packet_id();
            let packet_id_2 = client.assign_packet_id();
            assert_ne!(packet_id, packet_id_2);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_si_el_id_de_paquete_es_distinto_de_cero() {
        let args = vec!["127.0.0.1".to_string(), "8080".to_string()];
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => panic!("Error creating broker: {:?}", e),
        };

        let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
        let server_ready_clone = server_ready.clone();
        thread::spawn(move || {
            {
                let (lock, cvar) = &*server_ready_clone;
                let mut ready = lock.lock().unwrap();
                *ready = true;
                cvar.notify_all();
            }
            let _ = broker.server_run();
        });

        // Wait for the server to start
        {
            let (lock, cvar) = &*server_ready;
            let mut ready = lock.lock().unwrap();
            while !*ready {
                ready = cvar.wait(ready).unwrap();
            }
        }
        let handle = thread::spawn(move || {
            let connect_config =
                client_message::Connect::read_connect_config("src/drones/connect_config.json")
                    .unwrap();
            let (_, rx) = mpsc::channel();
            let (tx2, _) = mpsc::channel();
            let address = "127.0.0.1:8080";
            let client = Client::new(rx, address.to_string(), connect_config, tx2).unwrap();
            let packet_id = client.assign_packet_id();
            assert_ne!(packet_id, 0);
        });
        handle.join().unwrap();
    }
}
