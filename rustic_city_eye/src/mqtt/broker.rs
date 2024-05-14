use std::{
    collections::HashMap, net::{TcpListener, TcpStream}, sync::{Arc, Mutex}
};

use crate::mqtt::{
    broker_message::BrokerMessage, client_message::ClientMessage, protocol_error::ProtocolError, topic::{self, Topic}
};

static SERVER_ARGS: usize = 2;

/// El broker puede ser considerado un servidor.
/// Bindea a un puerto dado y comienza a correr.
/// Contiene la info de los mensajes que están siendo enviados y del cliente
/// Es un mediador entra los diferentes clientes, enviendo los ACKs correspondientes a los publishers. Esto permite implementar un tipo de comunicación asincronica.
#[derive(Clone)]
pub struct Broker {
    address: String,
    packet_ids: Vec<u16>,
    subscribers: Arc<Mutex<Vec<TcpStream>>>,
    topics: HashMap<String, Topic>
}

impl Broker {
    ///Chequea que el numero de argumentos sea valido.
    pub fn new(args: Vec<String>) -> Result<Broker, ProtocolError> {
        if args.len() != SERVER_ARGS {
            let app_name = &args[0];
            println!("Usage:\n{:?} <puerto>", app_name);
            return Err(ProtocolError::InvalidNumberOfArguments);
        }

        let packet_ids = Vec::new();
        let address = "127.0.0.1:".to_owned() + &args[1];
        let mut topics = HashMap::new();

        topics.insert("accidente".to_string(), Topic::new());

        Ok(Broker {
            address,
            packet_ids,
            subscribers: Arc::new(Mutex::new(Vec::new())),
            topics
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
                    let mut self_clone = self.clone(); // Clone the `self` reference
                    std::thread::spawn(move || {
                        let stream = Arc::new(Mutex::new(stream));
                        let _ = self_clone.handle_client(stream); // Use the cloned reference
                    });
                }
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }

    ///Se encarga del manejo de los mensajes del cliente. Envia los ACKs correspondientes.
    pub fn handle_client(
        &mut self,
        stream: Arc<Mutex<TcpStream>>,
    ) -> std::io::Result<()> {
        let stream_reference = Arc::clone(&stream);
        let mut stream = stream.lock().unwrap();

        while let Ok(message) = ClientMessage::read_from(&mut *stream) {
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
                    println!("Recibí un connect: {:?}", message);
                    let connack = BrokerMessage::Connack {
                        //session_present: true,
                        //return_code: 0,
                    };
                    println!("Sending connack: {:?}", connack);
                    connack.write_to(&mut *stream).unwrap();
                }
                ClientMessage::Publish {
                    packet_id,
                    topic_name: ref topic,
                    qos: _,
                    retain_flag: _,
                    payload: _,
                    dup_flag: _,
                    properties: _,
                } => {
              //      println!("topic {:?}", topic);
                    let packet_id_bytes: [u8; 2] = packet_id.to_be_bytes();
                    // ...
                    if topic == "accidente" {
                       // let delivery_message = message;

                        //delivery_message.write_to(&mut *stream)?;
                        //if let Some(topic) = self.topics.get_mut("accidente") {
                            // println!("mandando mensajes a mis subs");
                            // <topic::Topic as Clone>::clone(&topic).send_message(message)?;
                       // }
                      //  let subscribers = self.subscribers.lock().unwrap();

                        //for mut subscriber in self.subscribers.iter() {
                       //  println!("message {:?}", message);
                     //       let _ = message.write_to(&mut subscriber);
                            // println!("Sending message to subscriber");
                            // // Write the message to the subscriber's stream.
                            // // You would replace this with your actual message sending code.
                            // let mensaje = "accidente";
                            // let lenght = mensaje.len();
                            // let lenght_bytes = lenght.to_be_bytes();
                            // let mensaje_bytes = mensaje.as_bytes();
                            // subscriber.write_all(&lenght_bytes)?;

                            // subscriber.write_all(mensaje_bytes)?;
                        //}
                        let puback = BrokerMessage::Puback {
                            packet_id_msb: packet_id_bytes[0],
                            packet_id_lsb: packet_id_bytes[1],
                            reason_code: 1,
                        };
                        puback.write_to(&mut *stream)?;
                    }
                }
                ClientMessage::Subscribe {
                    packet_id: _,
                    topic_name: ref topic,
                    properties: _,
                } => {
                    //habria que agarrar el packet id y partirlo(porque el suback usa el mismo ;) ) 

                    println!("Recibí un subscribe: {:?}", message);
                    if topic == "accidente" {
                        // if let Some(topic) = self.topics.get_mut("accidente") {
                        //     topic.add_subscriber(stream_reference.clone());
                        // }
                        // let subs = self.subscribers.lock().unwrap();
                        // subs.push(stream);
                        println!("topic subs: {:?}", self.subscribers);
                        // let stream_reference = Arc::new(stream);
                        // let stream_clone = Arc::clone(&stream_reference);


                        
                      //  let subs_reference = Arc::clone(&self.subscribers);
                        //let mut subscribers = subs_reference.lock().unwrap();
                        
                        // match stream.try_clone(){
                        //     Ok(stream) => subscribers.push(stream),
                        //     Err(err) => println!("Error al clonar el stream: {:?}", err)
                        // };
                        
                     //   subscribers.push(stream.try_clone().unwrap());
                        // self.subscribers.push(stream.try_clone().unwrap());
                        // println!("subs del broker: {:?}", self.subscribers);

                       // println!("subs: {:?}", subscribers);
                        let suback = BrokerMessage::Suback {
                            packet_id_msb: 0,
                            packet_id_lsb: 1,
                            reason_code: 0,
                        };
                        println!("Sending suback: {:?}", suback);
                        match suback.write_to(&mut *stream){
                            Ok(_) => println!("suback enviado"),
                            Err(err) => println!("Error al enviar suback: {:?}", err)
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn assign_new_packet_id(&mut self) -> u16 {
        let mut new_id = rand::random::<u16>();

        while new_id == 0x00 || self.packet_ids.contains(&new_id) {
            new_id = rand::random::<u16>();
        }

        self.packet_ids.push(new_id);
        new_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_invalid_number_of_arguments_err() -> std::io::Result<()> {
        let mut args = Vec::new();
        args.push("target/debug/broker".to_string());

        let _ = Broker::new(args);

        Ok(())
    }
}
