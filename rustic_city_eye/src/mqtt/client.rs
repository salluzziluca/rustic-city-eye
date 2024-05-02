use std::net::TcpStream;

use crate::mqtt::broker_message::BrokerMessage;
use crate::mqtt::client_message::ClientMessage;
use crate::mqtt::protocol_error::ProtocolError;

use crate::mqtt::publish_properties::PublishProperties;

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

        //message interface(temp): dup:1 qos:2 retain:1 topic_name:sometopic 
        let mut dup_flag = false;
        let mut qos = 0;
        let mut retain_flag = false;
        let mut packet_id = 0x00;

        if splitted_message[0] == "dup:1" {
            dup_flag = true;
        }

        if splitted_message[1] == "qos:1" {
            qos = 1;
            packet_id = 0x20;
        }

        if splitted_message[2] == "retain:1" {
            retain_flag = true;
        }

        let properties = PublishProperties::new(1, 10, 10, "String".to_string(), [1, 2, 3].to_vec(), "a".to_string(), 1, "a".to_string());


        let publish = ClientMessage::Publish {
            packet_id,
            topic_name: splitted_message[3].to_string(),
            qos,
            retain_flag,
            payload: splitted_message[4].to_string(),
            dup_flag,
            properties
        };

        publish.write_to(&mut self.stream).unwrap();

        if let Ok(message) = BrokerMessage::read_from(&mut self.stream) {
            match message {
                BrokerMessage::Puback { reason_code: _
                } => {
                    println!("Recibí un puback: {:?}", message);
                },
                _ => println!("no recibi nada :("),

            }
        }
    }
}