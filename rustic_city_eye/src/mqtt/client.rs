use std::net::TcpStream;

use crate::mqtt::broker_message::BrokerMessage;
use crate::mqtt::client_message::ClientMessage;
use crate::mqtt::connect_propierties::ConnectProperties;
use crate::mqtt::protocol_error::ProtocolError;

use crate::mqtt::publish_properties::PublishProperties;
use crate::mqtt::will_properties::WillProperties;

#[allow(dead_code)]
pub struct Client {
    stream: TcpStream,
}
#[allow(dead_code)]
impl Client {
    pub fn new(address: &str) -> Result<Client, ProtocolError> {
        let mut stream = match TcpStream::connect(address) {
            Ok(stream) => stream,
            Err(_) => return Err(ProtocolError::ConectionError),
        };

        let will_properties = WillProperties::new(
            120,
            1,
            30,
            "plain".to_string(),
            "topic".to_string(),
            vec![1, 2, 3, 4, 5],
            vec![("propiedad".to_string(), "valor".to_string())],
        );

        let properties = ConnectProperties {
            session_expiry_interval: 10,
            receive_maximum: 10,
            maximum_packet_size: 10,
            topic_alias_maximum: 10,
            request_response_information: true,
            request_problem_information: true,
            user_properties: vec![("a".to_string(), "a".to_string())],
            authentication_method: "a".to_string(),
            authentication_data: vec![1, 2, 3, 4, 5],
        };

        let connect = ClientMessage::Connect {
            clean_start: true,
            last_will_flag: true,
            last_will_qos: 1,
            last_will_retain: true,
            username: "prueba".to_string(),
            password: "".to_string(),
            keep_alive: 35,
            properties,
            client_id: "kvtr33".to_string(),
            will_properties,
            last_will_topic: "topic".to_string(),
            last_will_message: "chauchis".to_string(),
        };
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
        } else {
            println!("soy el client y no pude leer el mensaje");
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

        let properties = PublishProperties::new(
            1,
            10,
            10,
            "String".to_string(),
            [1, 2, 3].to_vec(),
            "a".to_string(),
            1,
            "a".to_string(),
        );

        let publish = ClientMessage::Publish {
            packet_id,
            topic_name: splitted_message[3].to_string(),
            qos,
            retain_flag,
            payload: splitted_message[4].to_string(),
            dup_flag,
            properties,
        };
        let _ = message;

        publish.write_to(&mut self.stream).unwrap();

        if let Ok(message) = BrokerMessage::read_from(&mut self.stream) {
            match message {
                BrokerMessage::Puback { reason_code: _ } => {
                    println!("Recibí un puback: {:?}", message);
                }
                _ => println!("no recibi nada :("),
            }
        }
    }
}
