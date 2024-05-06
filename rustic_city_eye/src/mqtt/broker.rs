use std::net::{TcpListener, TcpStream};

use crate::mqtt::{broker_message::BrokerMessage, client_message::ClientMessage, protocol_error:: ProtocolError};
// use crate::mqtt::client_message::ClientMessage;
// use crate::mqtt::protocol_error::ProtocolError;

static SERVER_ARGS: usize = 2;

/// El broker puede ser considerado un sv.
/// Bindea a un puerto dado y comienza a correr.
/// Contiene la info de los mensajes que están siendo enviados y del cliente
/// Es un mediador entra los diferentes clientes, enviendo los ACKs correspondientes a los publishers. Esto permite implementar un tipo de comunicación asincronica.
pub struct Broker {
    address: String,
    packet_ids: Vec<u16>,
}

impl Broker {
    ///Check that the number of arguments is valid.
    pub fn new(args: Vec<String>) -> Result<Broker, ProtocolError> {
        if args.len() != SERVER_ARGS {
            let app_name = &args[0];
            println!("Usage:\n{:?} <puerto>", app_name);
            return Err(ProtocolError::InvalidNumberOfArguments);
        }

        let packet_ids = Vec::new();
        let address = "127.0.0.1:".to_owned() + &args[1];

        Ok(Broker {
            address,
            packet_ids,
        })
    }

    /// Ejecuta el servidor.
    /// Crea un enlace en la dirección del broker y, para
    /// cada conexión entrante, crea un hilo para manejar el nuevo cliente.
    pub fn server_run(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.address)?;

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    std::thread::spawn(move || Broker::handle_client(&mut stream));
                }
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }

    ///Se encarga del manejo de los mensajes del cliente. Envia los ACKs correspondientes.
    fn handle_client(stream: &mut TcpStream) -> std::io::Result<()> {
        while let Ok(message) = ClientMessage::read_from(stream) {
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
                    connack.write_to(stream).unwrap();
                }
                ClientMessage::Publish {
                    packet_id,
                    topic_name: _,
                    qos,
                    retain_flag: _,
                    payload: _,
                    dup_flag: _,
                    properties: _,
                } => {
                    println!("Recibí un publish: {:?}", message);
                    let packet_id_bytes: [u8; 2] = packet_id.to_be_bytes();

                    if qos == 1 {
                        println!("sending puback...");
                        let puback = BrokerMessage::Puback {
                            packet_id_msb: packet_id_bytes[0],
                            packet_id_lsb: packet_id_bytes[1],
                            reason_code: 1,
                        };
                        puback.write_to(stream)?;
                    }
                }
                ClientMessage::Subscribe {
                    packet_id: _,
                    topic_name: _,
                    properties: _,
                } => {
                    println!("Recibí un subscribe: {:?}", message);
                    let suback = BrokerMessage::Suback {
                        packet_id_msb: 0,
                        packet_id_lsb: 1,
                        reason_code: 0,
                    };
                    println!("Sending suback: {:?}", suback);
                    suback.write_to(stream).unwrap();
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

// fn main() -> Result<(), ProtocolError> {
//     let argv = args().collect::<Vec<String>>();
//     let broker = Broker::new(argv)?;
//     let _ = broker.server_run();
//     Ok(())
// }

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
