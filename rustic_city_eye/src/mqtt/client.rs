use std::net::TcpStream;

use crate::mqtt::broker_message::BrokerMessage;
use crate::mqtt::client_message::ClientMessage;
use crate::mqtt::connect_propierties::ConnectProperties;
use crate::mqtt::protocol_error::ProtocolError;
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

        let properties = ConnectProperties{ 
            session_expiry_interval: todo!(), 
            receive_maximum: todo!(), 
            maximum_packet_size: todo!(), 
            topic_alias_maximum: todo!(), 
            request_response_information: todo!(), 
            request_problem_information: todo!(), 
            user_properties: todo!(), 
            authentication_method: todo!(), 
            authentication_data: todo!(),
        };

        let connect = ClientMessage::Connect {
            clean_start: true,
            last_will_flag: true,
            last_will_qos: 1,
            last_will_retain: true,
            username: "prueba".to_string(),
            password: "".to_string(),
            keep_alive: 35,
            // properties: properties,
            client_id: "kvtr33".to_string(),
            will_properties: will_properties,
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
                }
            }
        } else {
            println!("soy el client y no pude leer el mensaje");
        }

        Ok(Client { stream })
    }

    pub fn publish_message(&mut self, message: &str) {
        let publish = ClientMessage::Publish {
            // packet_id: 1,
            // topic_name: "juan".to_string(),
            // qos: 0,
            // retain_flag: true,
            
            // dup_flag: true,
        };
        let _ = message;

        let _ = publish.write_to(&mut self.stream);
    }
}
