use rand::Rng;
use std::{
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

use super::{client_message, client_return::ClientReturn, error::ClientError, messages_config};

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
    stream: Arc<Mutex<TcpStream>>,

    // las subscriptions es un vector de topics a los que el cliente está subscrito
    pub subscriptions: Arc<Mutex<Vec<String>>>,

    // client_id es el identificador del cliente
    pub client_id: String,

    pub packets_ids: Arc<Mutex<Vec<u16>>>,

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
        let mut stream = match TcpStream::connect(address) {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::ConectionError),
        };

        let connect_message = ClientMessage::Connect(connect.clone());

        println!("Enviando connect message to broker");

        match connect_message.write_to(&mut stream) {
            Ok(()) => println!("Connect message enviado"),
            Err(_) => println!("Error al enviar connect message"),
        }

        if let Ok(message) = BrokerMessage::read_from(&mut stream) {
            match message {
                BrokerMessage::Connack {
                    session_present: _,
                    reason_code,
                    properties: _,
                } => {
                    println!("Recibí un Connack");
                    match reason_code {
                        0x00_u8 => {
                            println!("Conexion exitosa!");

                            let client_id = connect.client_id;
                            Ok(Client {
                                receiver_channel: Arc::new(Mutex::new(receiver_channel)),
                                stream: Arc::new(Mutex::new(stream)),
                                subscriptions: Arc::new(Mutex::new(Vec::new())),
                                packets_ids: Arc::new(Mutex::new(Vec::new())),
                                sender_channel: Some(sender_channel),
                                client_id,
                            })
                        }
                        _ => {
                            println!("Connack con reason code {}", reason_code);
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

    /// Publica un mensaje en un topic determinado.
    pub fn publish_message(
        &self,
        message: ClientMessage,
        packet_id: u16,
    ) -> Result<u16, ClientError> {
        if let ClientMessage::Publish {
            packet_id: _,
            topic_name,
            qos: _,
            retain_flag: _,
            payload: _,
            dup_flag: _,
            properties: _,
        } = message.clone()
        {
            {
                let mut lock = self
                    .stream
                    .lock()
                    .map_err(|_| ClientError::new("Error al adquirir el lock"))?;
                match message.write_to(&mut *lock) {
                    Ok(()) => {
                        println!("envio publish con topic_name: {:?}", topic_name);
                        Ok(packet_id)
                    }
                    Err(_) => Err(ClientError::new("Error al enviar mensaje")),
                }
            }
        } else {
            Err(ClientError::new("El mensaje no es de tipo publish"))
        }
    }

    pub fn subscribe(
        &self,
        message: ClientMessage,
        packet_id: u16,
        // mut stream: TcpStream,
    ) -> Result<u16, ClientError> {
        {
            let mut lock = self
                .stream
                .lock()
                .map_err(|_| ClientError::new("Error al adquirir el lock"))?;
            match message.write_to(&mut *lock) {
                Ok(()) => Ok(packet_id),
                Err(_) => Err(ClientError::new("Error al enviar mensaje")),
            }
        }
    }

    pub fn unsubscribe(
        &self,
        message: ClientMessage,
        // mut stream: TcpStream,
        packet_id: u16,
    ) -> Result<u16, ClientError> {
        {
            let mut lock = self
                .stream
                .lock()
                .map_err(|_| ClientError::new("Error al adquirir el lock"))?;
            match message.write_to(&mut *lock) {
                Ok(()) => Ok(packet_id),
                Err(_) => Err(ClientError::new("Error al enviar mensaje")),
            }
        }
    }

    ///recibe un string que indica la razón de la desconexión y un stream y envia un disconnect message al broker
    /// segun el str reason recibido, modifica el reason_code y el reason_string del mensaje
    ///
    /// devuelve el packet_id del mensaje enviado o un ClientError en caso de error
    pub fn handle_disconnect(
        &self,
        client_id: String,
        reason: &str,
        // mut stream: TcpStream,
    ) -> Result<(), ClientError> {
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
        let disconnect = ClientMessage::Disconnect {
            reason_code,
            session_expiry_interval: 0,
            reason_string,
            client_id,
        };

        {
            let mut lock = self
                .stream
                .lock()
                .map_err(|_| ClientError::new("Error al adquirir el lock"))?;
            match disconnect.write_to(&mut *lock) {
                Ok(()) => Ok(()),
                Err(_) => Err(ClientError::new("Error al enviar mensaje")),
            }
        }

        // match disconnect.write_to(&mut stream) {
        //     Ok(()) => Ok(()),
        //     Err(_) => Err(ClientError::new("Error al enviar mensaje")),
        // }
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

        let (write_sender, recieve_receiver) = mpsc::channel();
        let (reciever_sender, write_receiver) = mpsc::channel();

        //let receiver_channel = self.receiver_channel.clone();

        //let desconectar = false;
        // let stream_lock = match self.stream.lock() {
        //     Ok(stream) => stream,
        //     Err(_) => return Err(ProtocolError::StreamError),
        // };
        let sender_channel_clone = self.sender_channel.clone();

        // let stream_clone_one = match self.stream.try_clone() {
        //     Ok(stream) => stream,
        //     Err(_) => return Err(ProtocolError::StreamError),
        // };
        // let stream_clone_two = match self.stream.try_clone() {
        //     Ok(stream) => stream,
        //     Err(_) => return Err(ProtocolError::StreamError),
        // };
        // let stream_clone_three = match self.stream.try_clone() {
        //     Ok(stream) => stream,
        //     Err(_) => return Err(ProtocolError::StreamError),
        // };

        //let subscriptions_clone = self.subscriptions.clone();
        let (disconnect_sender, disconnect_receiver) = mpsc::channel();
        //let client_id_clone = self.client_id.clone();

        let self_ref = Arc::new(self.clone());
        let self_ref_two = Arc::new(self.clone());

        let _write_messages = {
          //  let cloned_self = Arc::clone(&self_ref);
            threadpool.execute(move || {
                self_ref.write_messages(
                    //   stream_clone_one,
                    // receiver_channel,
                    //desconectar,
                    write_sender,
                    // subscriptions_clone,
                    write_receiver,
                    disconnect_receiver,
                )
            })
        };

        if let Some(sender_channel) = sender_channel_clone {
            let _read_messages = {
                // let cloned_self = Arc::clone(&self_ref);
                threadpool.execute(move || {
                    match self_ref_two.receive_messages(
                        // stream_clone_two,
                        recieve_receiver,
                        reciever_sender,
                        sender_channel,
                        disconnect_sender,
                        // client_id_clone,
                    ) {
                        Ok(_) => {
                            // stream_clone_three.shutdown(Shutdown::Both).map_err(|_| {
                            //     ProtocolError::ShutdownError(
                            //         "Error al cerrar el stream".to_string(),
                            //     )
                            // })?;
                            Ok(())
                        }
                        Err(e) => Err(e),
                    }
                })
            };
        }
        Ok(())
    }

    pub fn receive_messages(
        &self,
        //stream: TcpStream,
        receiver: Receiver<u16>,
        sender: Sender<bool>,
        sender_channel: Sender<ClientMessage>,
        disconnect_sender: Sender<bool>,
        //   client_id: String,
    ) -> Result<(), ProtocolError> {
        let mut pending_messages = Vec::new();

        loop {
            if let Ok(packet) = receiver.try_recv() {
                if !pending_messages.contains(&packet) {
                    pending_messages.push(packet);
                }
            }

            //if let Ok(stream_clone) = stream.try_clone() {
            match self.handle_message(
                // stream_clone,
                pending_messages.clone(),
                sender.clone(),
                sender_channel.clone(),
                self.client_id.clone(),
            ) {
                Ok(return_val) => {
                    if return_val == ClientReturn::DisconnectRecieved {
                        disconnect_sender.send(true).expect("Error al desconectar");
                        return Ok(());
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
            // } else {
            //   println!("Error al clonar el stream");
            //}
        }
    }

    pub fn disconnect_client(&mut self) -> Result<(), ProtocolError> {
        self.sender_channel = None;
        let lock = match self.stream.lock() {
            Ok(lock) => lock,
            Err(_) => return Err(ProtocolError::StreamError),
        };

        match lock.shutdown(Shutdown::Both) {
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
        &self,
        //stream: TcpStream,
        // receiver_channel: Arc<Mutex<Receiver<Box<dyn MessagesConfig + Send>>>>,
        // mut desconectar: bool,
        sender: Sender<u16>,
        // subscriptions_clone: Arc<Mutex<Vec<String>>>,
        receiver: Receiver<bool>,
        disconnect_receiver: Receiver<bool>,
    ) -> Result<(), ProtocolError> {
        // while !desconectar {
        loop {
            if let Ok(disconnect_status) = disconnect_receiver.try_recv() {
                if disconnect_status {
                    return Ok(());
                }
            }

            let message = self.build_client_message()?;

            match message {
            //     ClientMessage::Publish {
            //         packet_id,
            //         topic_name,
            //         qos,
            //         retain_flag,
            //         payload,
            //         dup_flag,
            //         properties,
            //     } => {
            //         let publish = ClientMessage::Publish {
            //             packet_id,
            //             topic_name: topic_name.clone(),
            //             qos,
            //             retain_flag,
            //             payload: payload.clone(),
            //             dup_flag,
            //             properties: properties.clone(),
            //         };

            //         if let Ok(packet_id) = self.publish_message(
            //             publish,
            //             // match self.stream.try_clone() {
            //             //     Ok(stream) => stream,
            //             //     Err(_) => return Err(ProtocolError::StreamError),
            //             // },
            //             packet_id,
            //         ) {
            //             if qos == 1 {
            //                 // let stream_clone = match self.stream.try_clone() {
            //                 //     Ok(stream) => stream,
            //                 //     Err(_) => return Err(ProtocolError::StreamError),
            //                 // };
            //                 match sender.send(packet_id) {
            //                     Ok(_) => {
            //                         if let Ok(puback_recieved) = receiver.try_recv() {
            //                             if !puback_recieved {
            //                                 //reenviar el msj con un dup_flag + 1

            //                                 thread::sleep(Duration::from_millis(500));
            //                                 let publish = ClientMessage::Publish {
            //                                     packet_id,
            //                                     topic_name,
            //                                     qos,
            //                                     retain_flag,
            //                                     payload,
            //                                     dup_flag: dup_flag + 1,
            //                                     properties,
            //                                 };
            //                                 match self.publish_message(
            //                                     publish, // stream_clone,
            //                                     packet_id,
            //                                 ) {
            //                                     Ok(_) => {
            //                                         println!("Reenviando mensaje con dup_flag + 1")
            //                                     }
            //                                     Err(_) => {
            //                                         println!("Error al reenviar mensaje")
            //                                     }
            //                                 }
            //                             }
            //                         }
            //                     }
            //                     Err(_) => {
            //                         println!("Error al enviar packet_id de puback al receiver")
            //                     }
            //                 }
            //             }
            //         }
            //     }
                ClientMessage::Subscribe {
                    packet_id,
                    properties,
                    payload,
                } => {
                    let subscribe = ClientMessage::Subscribe {
                        packet_id,
                        properties,
                        payload: payload.clone(),
                    };
                    println!("toy");
                    //if let Ok(stream) = self.stream.try_clone() {
                    if let Ok(packet_id) = self.subscribe(subscribe, packet_id) {
                        match sender.send(packet_id) {
                            Ok(_) => {
                                let topic_new = payload.topic.to_string();
                                match self.subscriptions.lock() {
                                    Ok(mut guard) => {
                                        guard.push(topic_new);
                                    }
                                    Err(_) => {
                                        return Err::<(), ProtocolError>(
                                            ProtocolError::StreamError,
                                        );
                                    }
                                }
                            }
                            Err(_) => {
                                return Err::<(), ProtocolError>(ProtocolError::StreamError);
                            }
                        }
                    }
                    // } else {
                    //   return Err::<(), ProtocolError>(ProtocolError::StreamError);
                    //}
                }
            //     ClientMessage::Unsubscribe {
            //         packet_id,
            //         properties,
            //         payload,
            //     } => {
            //         let unsubscribe = ClientMessage::Unsubscribe {
            //             packet_id,
            //             properties,
            //             payload: payload.clone(),
            //         };

            //         // if let Ok(stream) = self.stream.try_clone() {
            //         if let Ok(packet_id) = self.unsubscribe(unsubscribe, packet_id) {
            //             match sender.send(packet_id) {
            //                 Ok(_) => {
            //                     let topic_new = payload.topic.to_string();
            //                     match self.subscriptions.lock() {
            //                         Ok(mut guard) => {
            //                             guard.retain(|x| x != &topic_new);
            //                         }
            //                         Err(_) => {
            //                             return Err::<(), ProtocolError>(
            //                                 ProtocolError::StreamError,
            //                             );
            //                         }
            //                     }
            //                 }
            //                 Err(_) => {
            //                     return Err::<(), ProtocolError>(ProtocolError::StreamError);
            //                 }
            //             }
            //         }
            //         // } else {
            //         //      return Err::<(), ProtocolError>(ProtocolError::StreamError);
            //         //  }
            //     }
            //     ClientMessage::Disconnect {
            //         reason_code: _,
            //         session_expiry_interval: _,
            //         reason_string: _,
            //         client_id,
            //     } => {
            //         //  match self.stream.try_clone() {
            //         // Ok(stream_clone) => {
            //         let reason = "normal";
            //         if self.handle_disconnect(client_id, reason).is_ok() {
            //             break;
            //         }

            //         // if let Ok(()) =
            //         //     self.handle_disconnect(client_id, reason)
            //         // {
            //         //     //  desconectar = true;
            //         //     break;
            //         // }
            //         //}
            //         //   Err(_) => {
            //         //       return Err::<(), ProtocolError>(ProtocolError::StreamError);
            //         //   }
            //         //  }
            //         // break;
            //     }
            //     ClientMessage::Pingreq => {
            //         //   match self.stream.try_clone() {
            //         //  Ok(mut stream_clone) => {
            //         let pingreq = ClientMessage::Pingreq;
            //         {
            //             let mut lock = self.stream.lock().unwrap();
            //             match pingreq.write_to(&mut *lock) {
            //                 Ok(()) => println!("Pingreq enviado"),
            //                 Err(_) => println!("Error al enviar pingreq"),
            //             }
            //         }
            //         //    }
            //         //    Err(_) => {
            //         //        return Err::<(), ProtocolError>(ProtocolError::StreamError);
            //         //   }
            //         // }
            //     }
            //     ClientMessage::Auth {
            //         reason_code: _,
            //         authentication_method: _,
            //         authentication_data: _,
            //         reason_string: _,
            //         user_properties: _,
            //     } => {
            //         println!("auth enviado");
            //     }
                _ => {}
            }
            //}
        }
        Ok(())
    }

    fn build_client_message(&self) -> Result<ClientMessage, ProtocolError> {
        {
            let lock = match self.receiver_channel.lock() {
                Ok(lock) => lock,
                Err(_) => return Err(ProtocolError::LockError),
            };

            if let Ok(message_config) = lock.try_recv() {
                let packet_id = self.assign_packet_id();

                let message = message_config.parse_message(packet_id);
                println!("message {:?}", message);
                Ok(message)
            } else {
                Err(ProtocolError::NotReceivedMessageError)
            }
        }
    }

    /// Lee del stream un mensaje y lo procesa
    /// Devuelve un ClientReturn con informacion del mensaje recibido
    /// O ProtocolError en caso de error
    pub fn handle_message(
        &self,
        // mut stream: TcpStream,
        pending_messages: Vec<u16>,
        sender: Sender<bool>,
        sender_chanell: Sender<ClientMessage>,
        client_id: String,
    ) -> Result<ClientReturn, ProtocolError> {
        let message = self.read_message_from_broker()?;

        //if let Ok(message) = BrokerMessage::read_from(&mut stream) {
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
        /*
        } else {
         Err(ProtocolError::StreamError)
        }
        */
    }

    fn read_message_from_broker(&self) -> Result<BrokerMessage, ProtocolError> {
        {
            let mut lock = self.stream.lock().unwrap();
            match BrokerMessage::read_from(&mut *lock) {
                Ok(message) => Ok(message),
                Err(_) => return Err(ProtocolError::StreamError),
            }
        }
    }

    pub fn get_publish_end_channel(
        &self,
    ) -> Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Box<(dyn MessagesConfig + Send + 'static)>>>>
    {
        self.receiver_channel.clone()
    }

    ///Asigna un id random
    pub fn assign_packet_id(&self) -> u16 {
        let mut rng = rand::thread_rng();
        let mut packet_ids = match self.packets_ids.lock() {
            Ok(packet_ids) => packet_ids,
            Err(_) => return 0,
        };

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

impl ClientTrait for Client {
    fn client_run(&mut self) -> Result<(), ProtocolError> {
        self.client_run()
    }

    fn clone_box(&self) -> Box<dyn ClientTrait> {
        Box::new(self.clone())
    }
    fn assign_packet_id(&self) -> u16 {
        self.assign_packet_id()
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
        let lock = match self.stream.lock() {
            Ok(lock) => lock,
            Err(_) => return Err(ProtocolError::StreamError),
        };

        //let writer = &mut lock;

        // writer.flush().map_err(|_| ProtocolError::WriteError)?;

        match lock.shutdown(Shutdown::Both) {
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
    mut stream: TcpStream,
    pending_messages: Vec<u16>,
    sender: Sender<bool>,
    sender_chanell: Sender<ClientMessage>,
    client_id: String,
) -> Result<ClientReturn, ProtocolError> {
    if let Ok(message) = BrokerMessage::read_from(&mut stream) {
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
