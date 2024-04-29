use std::net::TcpStream;

use self::broker_message::BrokerMessage;
use self::client_message::ClientMessage;
use self::protocol_error::ProtocolError;

#[path = "broker_message.rs"]
mod broker_message;
#[path = "client_message.rs"]
mod client_message;
#[path = "protocol_error.rs"]
mod protocol_error;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(address: &str) -> Result<Client, ProtocolError> {
        let mut stream = match TcpStream::connect(address) {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::ConectionError),
        };

        let connect = ClientMessage::Connect {};
        println!("Sending connect message to broker: {:?}", connect);
        connect.write_to(&mut stream).unwrap();

        if let Ok(message) = BrokerMessage::read_from(&mut stream) {
            match message {
                BrokerMessage::Connack {
                   //session_present,
                    //return_code,
                } => {
                    println!("Recibí un connack: {:?}", message);
                },
                _ => println!("no recibi un connack :("),

            }
        }

        Ok(Client { stream })
    }

    pub fn publish_message(&mut self, message: &str) {
        let splitted_message: Vec<&str> = message.split(' ').collect();
        println!("{:?}", splitted_message);
        //message interface(temp): dup:1 qos:2 retain:1 topic_name:sometopic 
        let mut dup_flag = false;
        let mut qos = 0;
        let mut retain_flag = false;

        if splitted_message[0] == "dup:1" {
            dup_flag = true;
        }

        if splitted_message[1] == "qos:1" {
            qos = 1;
        } else if splitted_message[1] == "qos:2" {
            qos = 2;
        }

        if splitted_message[2] == "retain:1" {
            retain_flag = true;
        }

        let publish = ClientMessage::Publish {
            packet_id: 1,
            topic_name: splitted_message[3].to_string(),
            qos,
            retain_flag,
            payload: splitted_message[4].to_string(),
            dup_flag,
        };

        publish.write_to(&mut self.stream).unwrap();

        if let Ok(message) = BrokerMessage::read_from(&mut self.stream) {
            match message {
                BrokerMessage::Puback {
                } => {
                    println!("Recibí un puback: {:?}", message);
                },
                _ => println!("no recibi nada :("),

            }
        }
    }
}