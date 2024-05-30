use crate::{
    helpers::payload_types::PayloadTypes,
    mqtt::{
        client_message::ClientMessage, messages_config::MessagesConfig,
        publish_properties::PublishProperties,
    },
};

pub struct PublishConfig {
    pub(crate) dup_flag: usize,
    pub(crate) qos: usize,
    pub(crate) retain_flag: usize,
    pub(crate) topic_name: String,
    pub(crate) payload: PayloadTypes,
    pub(crate) publish_properties: PublishProperties,
}

impl MessagesConfig for PublishConfig {
    fn parse_message(&self, packet_id: u16) -> ClientMessage {
        ClientMessage::Publish {
            packet_id,
            topic_name: self.topic_name.clone(),
            qos: self.qos,
            retain_flag: self.retain_flag,
            payload: self.payload.clone(),
            dup_flag: self.dup_flag,
            properties: self.publish_properties.clone(),
        }
    }
}

impl PublishConfig {
    pub fn new(
        dup_flag: usize,
        qos: usize,
        retain_flag: usize,
        topic_name: String,
        payload: PayloadTypes,
        publish_properties: PublishProperties,
    ) -> PublishConfig {
        PublishConfig {
            dup_flag,
            qos,
            retain_flag,
            topic_name,
            payload,
            publish_properties,
        }
    }
}
