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
