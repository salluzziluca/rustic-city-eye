use crate::mqtt::subscribe_properties::SubscribeProperties;

use super::{
    client_message::ClientMessage, messages_config::MessagesConfig, subscription::Subscription,
};

pub struct SubscribeConfig {
    pub(crate) topic_name: String,
    pub(crate) qos: u8,
    pub(crate) properties: SubscribeProperties,
}

impl MessagesConfig for SubscribeConfig {
    fn parse_message(&self, packet_id: u16) -> ClientMessage {
        let subscription = Subscription::new(
            self.topic_name.clone(),
            "juancito".to_string(),
            self.qos.clone(),
        );
        //creo un vector cno la subscription
        let mut subscriptions = Vec::new();
        subscriptions.push(subscription);

        ClientMessage::Subscribe {
            packet_id,
            properties: self.properties.clone(),
            payload: subscriptions,
        }
    }
}

impl SubscribeConfig {
    pub fn new(topic_name: String, qos: u8, properties: SubscribeProperties) -> SubscribeConfig {
        SubscribeConfig {
            topic_name,
            qos,
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
        let properties =
            SubscribeProperties::new(1, vec![("key".to_string(), "value".to_string())]);
        let qos = 1;
        let subscribe_config = SubscribeConfig::new(topic_name.clone(), qos, properties.clone());
        assert_eq!(subscribe_config.topic_name, topic_name);
        assert_eq!(subscribe_config.qos, qos);
        assert_eq!(subscribe_config.properties, properties);
    }

    #[test]
    fn test_parse_message() {
        let topic_name = "topic".to_string();
        let properties =
            SubscribeProperties::new(1, vec![("key".to_string(), "value".to_string())]);

        let qos = 1;
        let subscribe_config = SubscribeConfig::new(topic_name.clone(), qos, properties.clone());
        let packet_id = 1;
        let message = subscribe_config.parse_message(packet_id);
        match message {
            ClientMessage::Subscribe {
                packet_id: message_packet_id,
                properties: message_properties,
                payload,
            } => {
                assert_eq!(message_packet_id, packet_id);
                assert_eq!(message_properties, properties);
                assert_eq!(payload.len(), 1);
                assert_eq!(payload[0].topic, topic_name);
                assert_eq!(payload[0].qos, qos);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
