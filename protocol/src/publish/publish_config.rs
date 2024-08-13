use std::{fs::File, io::BufReader};

use serde::Deserialize;

use crate::{client_message::ClientMessage, messages_config::MessagesConfig};
use utils::protocol_error::ProtocolError;

use super::{payload_types::PayloadTypes, publish_properties::PublishProperties};

#[derive(Deserialize, Debug, Clone, PartialEq)]
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

    pub fn read_json_to_publish_config(payload: PayloadTypes, json_data: &str) -> PublishConfig {
        let config: PublishConfig = match serde_json::from_str(json_data) {
            Ok(config) => config,
            Err(e) => panic!("Error reading json to PublishConfig: {}", e),
        };

        PublishConfig {
            dup_flag: config.dup_flag,
            qos: config.qos,
            retain_flag: config.retain_flag,
            topic_name: config.topic_name,
            payload,
            publish_properties: config.publish_properties,
        }
    }
    /// Abre un archivo de configuracion con propiedades y guarda sus lecturas.
    pub fn read_config(
        file_path: &str,
        payload: PayloadTypes,
    ) -> Result<PublishConfig, ProtocolError> {
        let config_file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => return Err(ProtocolError::ReadingConfigFileError),
        };

        let reader: BufReader<File> = BufReader::new(config_file);
        let config: PublishConfig = match serde_json::from_reader(reader) {
            Ok(c) => c,
            Err(_) => return Err(ProtocolError::ReadingConfigFileError),
        };
        Ok(PublishConfig {
            dup_flag: config.dup_flag,
            qos: config.qos,
            retain_flag: config.retain_flag,
            topic_name: config.topic_name,
            payload,
            publish_properties: config.publish_properties,
        })
    }
}

#[cfg(test)]

mod tests {
    // use crate::publish::publish_properties::TopicProperties;

    //use super::*;
    // use crate::{
    //     monitoring::incident::Incident,
    //     mqtt::{client_message::ClientMessage, publish::publish_properties::TopicProperties},
    //     utils::{incident_payload, location::Location},
    // };

    // #[test]
    // fn test_parse_message() {
    //     let topic_properties = TopicProperties {
    //         topic_alias: 10,
    //         response_topic: "String".to_string(),
    //     };
    //     let location = Location::new(12.1, 25.0);
    //     let incident = Incident::new(location);
    //     let incident_payload = incident_payload::IncidentPayload::new(incident);
    //     let publish_prop = PublishProperties::new(
    //         1,
    //         10,
    //         topic_properties,
    //         [1, 2, 3].to_vec(),
    //         "a".to_string(),
    //         1,
    //         "a".to_string(),
    //     );
    //     let publish_config = PublishConfig::new(
    //         1,
    //         1,
    //         1,
    //         "topic".to_string(),
    //         PayloadTypes::IncidentLocation(incident_payload.clone()).clone(),
    //         publish_prop.clone(),
    //     );
    //     let client_message = publish_config.parse_message(1);
    //     assert_eq!(
    //         client_message,
    //         ClientMessage::Publish {
    //             packet_id: 1,
    //             topic_name: "topic".to_string(),
    //             qos: 1,
    //             retain_flag: 1,
    //             payload: PayloadTypes::IncidentLocation(incident_payload),
    //             dup_flag: 1,
    //             properties: publish_prop
    //         }
    //     );
    // }
}
