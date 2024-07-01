use serde::{Deserialize, Serialize};

use crate::mqtt::subscribe_properties::SubscribeProperties;

use super::{
    client_message::ClientMessage, messages_config::MessagesConfig, subscription::Subscription,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubscribeConfig {
    pub(crate) topic_name: String,
    pub(crate) properties: SubscribeProperties,
    pub(crate) client_id: String,
}

impl MessagesConfig for SubscribeConfig {
    fn parse_message(&self, packet_id: u16) -> ClientMessage {
        let payload = Subscription::new(self.topic_name.clone(), self.client_id.clone());
        //creo un vector cno la subscription

        ClientMessage::Subscribe {
            packet_id,
            properties: self.properties.clone(),
            payload,
        }
    }
}

impl SubscribeConfig {
    pub fn new(
        topic_name: String,
        properties: SubscribeProperties,
        client_id: String,
    ) -> SubscribeConfig {
        SubscribeConfig {
            topic_name,
            properties,
            client_id,
        }
    }

    pub fn json_to_publish_config(path: &str) -> SubscribeConfig {
        let config: SubscribeConfig = match serde_json::from_str(path) {
            Ok(config) => config,
            Err(e) => panic!("Error reading json to PublishConfig: {}", e),
        };

        SubscribeConfig {
            topic_name: config.topic_name,
            properties: config.properties,
            client_id: config.client_id,
        }
    }
    pub fn write_config_to_json_file(&self, path: &str) {
        let json = match serde_json::to_string(&self) {
            Ok(json) => json,
            Err(e) => panic!("Error converting PublishConfig to json: {}", e),
        };
        match std::fs::write(path, json) {
            Ok(_) => {}
            Err(e) => panic!("Error writing PublishConfig to json file: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_subscribe_config() {
        let topic_name = "topic".to_string();
        let properties =
            SubscribeProperties::new(1, vec![("key".to_string(), "value".to_string())]);
        let subscribe_config =
            SubscribeConfig::new(topic_name.clone(), properties.clone(), "client".to_string());
        assert_eq!(subscribe_config.topic_name, topic_name);
        assert_eq!(subscribe_config.properties, properties);
    }

    #[test]
    fn test_parse_message() {
        let topic_name = "topic".to_string();
        let properties =
            SubscribeProperties::new(1, vec![("key".to_string(), "value".to_string())]);

        let subscribe_config =
            SubscribeConfig::new(topic_name.clone(), properties.clone(), "client".to_string());
        let packet_id = 1;
        let message = subscribe_config.parse_message(packet_id);

        let payload_1 = Subscription::new(topic_name.clone(), "client".to_string());

        match message {
            ClientMessage::Subscribe {
                packet_id: message_packet_id,
                properties: message_properties,
                payload,
            } => {
                assert_eq!(message_packet_id, packet_id);
                assert_eq!(message_properties, properties);
                assert_eq!(payload_1, payload);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
