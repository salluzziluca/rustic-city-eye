use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Error;
use std::net::TcpStream;
use std::sync::{Arc, RwLock};

use crate::mqtt::broker_message::BrokerMessage;

use super::client_message::ClientMessage;

use super::reason_code;

#[derive(Debug, Clone)]
pub struct Topic {
    /// Hashmap de subscriptores.
    /// El u32 representa el sub_id, y el valor es el stream del subscriptor.
    subscribers: Arc<RwLock<HashMap<u8, TcpStream>>>,
}

impl Default for Topic {
    fn default() -> Self {
        Self::new()
    }
}

impl Topic {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_subscriber(&mut self, stream: TcpStream, sub_id: u8) -> u8 {
        //verificar si el sub_id ya existe en topic subscribers
        if self.subscribers.read().unwrap().contains_key(&sub_id) {
            return reason_code::SUB_ID_DUP_HEX;
        }

        let mut lock = match self.subscribers.write() {
            Ok(guard) => guard,
            Err(_) => return reason_code::UNSPECIFIED_ERROR_HEX,
        };

        lock.insert(sub_id, stream);
        reason_code::SUCCESS_HEX
    }

    pub fn remove_subscriber(&mut self, sub_id: u8) -> u8 {
        let mut lock = match self.subscribers.write() {
            Ok(guard) => guard,
            Err(_) => return reason_code::UNSPECIFIED_ERROR_HEX,
        };

        if !lock.contains_key(&sub_id) {
            println!("No se encontró el sub_id en los subscribers");
            return reason_code::NO_MATCHING_SUBSCRIBERS_HEX;
        }

        lock.remove(&sub_id);
        reason_code::SUCCESS_HEX
    }

    pub fn deliver_message(&self, message: ClientMessage) -> Result<u8, Error> {
        let lock = self.subscribers.read().unwrap();
        let mut puback_reason_code: u8 = 0x00;
        match message {
            ClientMessage::Publish {
                topic_name,
                payload,
                packet_id,
                qos,
                retain_flag,
                dup_flag,
                properties,
            } => {
                let delivery_message = BrokerMessage::PublishDelivery {
                    topic_name,
                    payload,
                    packet_id,
                    qos,
                    retain_flag,
                    dup_flag,
                    properties,
                };

                if lock.is_empty() {
                    puback_reason_code = 0x10_u8;
                    return Ok(puback_reason_code);
                }

                for mut subscriber in lock.iter() {
                    println!("Enviando un PublishDelivery");
                    match delivery_message.write_to(&mut subscriber.1) {
                        Ok(_) => println!("PublishDelivery enviado"),
                        Err(err) => println!("Error al enviar PublishDelivery: {:?}", err),
                    }
                }
            }
            _ => {
                return Ok(0x10);
            }
        };
        Ok(puback_reason_code)
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;

    use crate::{
        monitoring::incident::Incident,
        mqtt::publish_properties::{PublishProperties, TopicProperties},
        utils::{incident_payload, location::Location, payload_types::PayloadTypes},
    };

    use super::*;

    #[test]
    fn test_new_topic() {
        let topic = Topic::new();
        assert_eq!(topic.subscribers.read().unwrap().len(), 0);
    }

    #[test]
    fn test_add_subscriber() {
        let mut topic = Topic::new();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let sub_id = 1;
        let result = topic.add_subscriber(stream, sub_id);
        assert_eq!(result, 0x00);
    }

    #[test]
    fn test_add_subscriber_duplicate() {
        let mut topic = Topic::new();
        let listener = TcpListener::bind("127.0.0.1:0");
        let stream = TcpStream::connect(listener.unwrap().local_addr().unwrap()).unwrap();
        let sub_id = 1;
        let result = topic.add_subscriber(stream.try_clone().unwrap(), sub_id);
        assert_eq!(result, 0x00);
        let result = topic.add_subscriber(stream, sub_id);
        assert_eq!(result, 0x85);
    }

    #[test]
    fn test_remove_subscriber() {
        let mut topic = Topic::new();
        let listener = TcpListener::bind("127.0.0.1:0");
        let stream = TcpStream::connect(listener.unwrap().local_addr().unwrap()).unwrap();
        let sub_id = 1;
        let result = topic.add_subscriber(stream, sub_id);
        assert_eq!(result, 0x00);
        let result = topic.remove_subscriber(sub_id);
        assert_eq!(result, 0x00);
    }

    #[test]
    fn test_remove_subscriber_no_matching() {
        let mut topic = Topic::new();
        let listener = TcpListener::bind("127.0.0.1:0");
        let stream = TcpStream::connect(listener.unwrap().local_addr().unwrap()).unwrap();

        let sub_id = 1;
        let result = topic.add_subscriber(stream, sub_id);
        assert_eq!(result, 0x00);
        let result = topic.remove_subscriber(2);
        assert_eq!(result, 0x10);
    }

    #[test]
    fn test_deliver_message() {
        let mut topic = Topic::new();
        let listener = TcpListener::bind("127.0.0.1:0");
        let stream = TcpStream::connect(listener.unwrap().local_addr().unwrap()).unwrap();
        let sub_id = 1;
        let result = topic.add_subscriber(stream, sub_id);
        let topic_properties = TopicProperties {
            topic_alias: 10,
            response_topic: "String".to_string(),
        };
        let properties = PublishProperties::new(
            1,
            10,
            topic_properties,
            [1, 2, 3].to_vec(),
            "a".to_string(),
            1,
            "a".to_string(),
        );
        assert_eq!(result, 0x00);
        let location = Location::new("1".to_string(), "1".to_string());
        let new = Incident::new(location);
        let incident_payload = incident_payload::IncidentPayload::new(new);
        let publish = ClientMessage::Publish {
            packet_id: 1,
            topic_name: "mensajes para juan".to_string(),
            qos: 1,
            retain_flag: 1,
            payload: PayloadTypes::IncidentLocation(incident_payload),
            dup_flag: 1,
            properties,
        };
        let result = match topic.deliver_message(publish) {
            Ok(result) => result,
            Err(err) => panic!("Error al enviar mensaje: {:?}", err),
        };
        assert_eq!(result, 0x00);
    }

    #[test]
    fn test_deliver_message_no_subscribers() {
        let topic = Topic::new();
        let topic_properties = TopicProperties {
            topic_alias: 10,
            response_topic: "String".to_string(),
        };
        let properties = PublishProperties::new(
            1,
            10,
            topic_properties,
            [1, 2, 3].to_vec(),
            "a".to_string(),
            1,
            "a".to_string(),
        );
        let location = Location::new("1".to_string(), "1".to_string());
        let new = Incident::new(location);
        let incident_payload = incident_payload::IncidentPayload::new(new);
        let publish = ClientMessage::Publish {
            packet_id: 1,
            topic_name: "mensajes para juan".to_string(),
            qos: 1,
            retain_flag: 1,
            payload: PayloadTypes::IncidentLocation(incident_payload),
            dup_flag: 1,
            properties,
        };
        let result = match topic.deliver_message(publish) {
            Ok(result) => result,
            Err(err) => panic!("Error al enviar mensaje: {:?}", err),
        };
        assert_eq!(result, 0x10);
    }
}
