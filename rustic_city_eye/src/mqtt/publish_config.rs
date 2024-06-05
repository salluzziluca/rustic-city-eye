use crate::{
    mqtt::{
        client_message::ClientMessage, messages_config::MessagesConfig,
        publish_properties::PublishProperties,
    },
    utils::payload_types::PayloadTypes,
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

#[cfg(test)]

mod tests {
    use super::*;
    use crate::{
        monitoring::incident::Incident,
        mqtt::{client_message::ClientMessage, publish_properties::TopicProperties},
        utils::{incident_payload, location::Location},
    };

    #[test]
    fn test_parse_message() {
        let topic_properties = TopicProperties {
            topic_alias: 10,
            response_topic: "String".to_string(),
        };
        let location = Location::new(12.1, 25.0);
        let incident = Incident::new(location);
        let incident_payload = incident_payload::IncidentPayload::new(incident);
        let publish_prop = PublishProperties::new(
            1,
            10,
            topic_properties,
            [1, 2, 3].to_vec(),
            "a".to_string(),
            1,
            "a".to_string(),
        );
        let publish_config = PublishConfig::new(
            1,
            1,
            1,
            "topic".to_string(),
            PayloadTypes::IncidentLocation(incident_payload.clone()).clone(),
            publish_prop.clone(),
        );
        let client_message = publish_config.parse_message(1);
        assert_eq!(
            client_message,
            ClientMessage::Publish {
                packet_id: 1,
                topic_name: "topic".to_string(),
                qos: 1,
                retain_flag: 1,
                payload: PayloadTypes::IncidentLocation(incident_payload),
                dup_flag: 1,
                properties: publish_prop
            }
        );
    }
}
