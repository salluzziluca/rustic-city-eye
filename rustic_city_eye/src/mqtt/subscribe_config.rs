use crate::mqtt::subscribe_properties::SubscribeProperties;

use super::{client_message::ClientMessage, messages_config::MessagesConfig};

pub struct SubscribeConfig {
    pub(crate) topic_name: String,
    pub(crate) properties: SubscribeProperties,
}

impl MessagesConfig for SubscribeConfig {
    fn parse_message(&self, packet_id: u16) -> ClientMessage {
        ClientMessage::Subscribe {
            packet_id,
            topic_name: self.topic_name.clone(),
            properties: self.properties.clone(),
        }
    }
}

impl SubscribeConfig {
    pub fn new(topic_name: String, properties: SubscribeProperties) -> SubscribeConfig {
        SubscribeConfig {
            topic_name,
            properties,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_subscribe_config() {
        let topic_name = "topic".to_string();
        let properties = SubscribeProperties::new(
            1,
            vec![("key".to_string(), "value".to_string())],
            vec![1, 2, 3, 4],
        );
        let subscribe_config = SubscribeConfig::new(topic_name.clone(), properties.clone());
        assert_eq!(subscribe_config.topic_name, topic_name);
        assert_eq!(subscribe_config.properties, properties);
    }

    #[test]
    fn test_parse_message() {
        let topic_name = "topic".to_string();
        let properties = SubscribeProperties::new(
            1,
            vec![("key".to_string(), "value".to_string())],
            vec![1, 2, 3, 4],
        );
        let subscribe_config = SubscribeConfig::new(topic_name.clone(), properties.clone());
        let packet_id = 1;
        let message = subscribe_config.parse_message(packet_id);
        match message {
            ClientMessage::Subscribe {
                packet_id: message_packet_id,
                topic_name: message_topic_name,
                properties: message_properties,
            } => {
                assert_eq!(message_packet_id, packet_id);
                assert_eq!(message_topic_name, topic_name);
                assert_eq!(message_properties, properties);
            }
            _ => panic!("Invalid message type"),
        }
    }
}
