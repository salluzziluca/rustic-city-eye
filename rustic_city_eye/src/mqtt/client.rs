use std::net::TcpStream;

use crate::mqtt::broker_message::BrokerMessage;
use crate::mqtt::client_message::ClientMessage;
use crate::mqtt::protocol_error::ProtocolError;

use crate::mqtt::subscribe_properties::SubscribeProperties;

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

        let connect = ClientMessage::Connect {
            clean_start: true,
            last_will_flag: true,
            last_will_qos: 1,
            last_will_retain: true,
            username: "prueba".to_string(),
            password: "".to_string(),
            keep_alive: 35,
            client_id: "kvtr33".to_string(),
            last_will_delay_interval: 15,
            //message_expiry_interval: 120,
            content_type: "plain".to_string(),
            user_property: Some(("propiedad".to_string(), "valor".to_string())),
            last_will_message: "me he muerto, diganle a mi vieja que la quiero, adios".to_string(),
            response_topic: "response_topic".to_string(),
            correlation_data: "correlation_data".to_string().into(),
        };
        //println!("Sending connect message to broker: {:?}", connect);
        println!("Sending connect message to broker");
        connect.write_to(&mut stream).unwrap();

        if let Ok(message) = BrokerMessage::read_from(&mut stream) {
            match message {
                BrokerMessage::Connack {
                   //session_present,
                    //return_code,
                } => {
                    println!("Recibí un connack: {:?}", message);
                },
                _ => {
                    println!("Recibí un mensaje que no es connack");
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

    pub fn subscribe(&mut self, topic: &str)  {
        println!("Entra a subscribe"); 
        let subscribe = ClientMessage::Subscribe {
            packet_id: 1,
            topic_name: topic.to_string(),
            properties: SubscribeProperties::new(
                1, 
                vec![("propiedad".to_string(), 
                "valor".to_string())], 
                vec![0, 1, 2, 3]),
            };

        subscribe.write_to(&mut self.stream).unwrap();

        println!("Después de write_to" );  //hasta acá llega

        if let Ok(message) = BrokerMessage::read_from(&mut self.stream) {
            match message {
                BrokerMessage::Suback {
                    reason_code: _,
                } => {
                    println!("Recibí un suback: {:?}", message);
                },
                _ => println!("Recibí un mensaje que no es suback"),
                
            }
        } else {
            println!("soy el client y no pude leer el mensaje 2"); // imprime esto
        }
    }
}
