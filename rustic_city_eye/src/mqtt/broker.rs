use std::env::args;
use std::net::{TcpListener, TcpStream};

use rustic_city_eye::mqtt::broker_message::BrokerMessage;
use rustic_city_eye::mqtt::client_message::ClientMessage;
use rustic_city_eye::mqtt::protocol_error::ProtocolError;

static SERVER_ARGS: usize = 2;

/// The broker can be thought of as the server.
/// Binds to a given port and starts running.
/// Contains information about the messages being sent and also about the clients.
/// Plays the role of mediator between different clients, and sends ACK messages to the publishers. This allows us to have an asynchronous communication.
pub struct Broker {
    address: String, 
    packet_ids: Vec<u16>
}

impl Broker {
    ///Check that the number of arguments is valid.
    pub fn build(args: Vec<String>) -> Result<Broker, ProtocolError> {
        if args.len() != SERVER_ARGS {
            let app_name = &args[0];
            println!("Usage:\n{:?} <puerto>", app_name);
            return Err(ProtocolError::InvalidNumberOfArguments);
        }

        let packet_ids = Vec::new();
        let address = "127.0.0.1:".to_owned() + &args[1];

        Ok(Broker { address, packet_ids})
    }

    ///Runs the server. 
    /// It creates a bind in the broker address, and for 
    /// every incoming connection creates a thread 
    /// to handle the new client.
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

    ///Handles the different client messages, and it sends the corresponding acknoledges.
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
                        let puback = BrokerMessage::Puback { packet_id_msb: packet_id_bytes[0], packet_id_lsb: packet_id_bytes[1], reason_code: 1 };
                        puback.write_to(stream)?;
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

fn main() -> Result<(), ProtocolError> {
    let argv = args().collect::<Vec<String>>();
    let broker = Broker::build(argv)?;
    let _ = broker.server_run();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_invalid_number_of_arguments_err() -> std::io::Result<()> {
        let mut args = Vec::new();
        args.push("target/debug/broker".to_string());

        let _ = Broker::build(args);

        Ok(())
    }
}