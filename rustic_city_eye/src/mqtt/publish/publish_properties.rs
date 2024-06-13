use std::io::{Error, Read, Write};

use crate::utils::{reader::*, writer::*};

use crate::mqtt::protocol_error::ProtocolError;

//PROPERTIES IDs
const PAYLOAD_FORMAT_INDICATOR_ID: u8 = 0x01;
const MESSAGE_EXPIRY_INTERVAL_ID: u8 = 0x02;
const TOPIC_ALIAS_ID: u8 = 0x23;
const RESPONSE_TOPIC_ID: u8 = 0x08;
const CORRELATION_DATA_ID: u8 = 0x09;
const USER_PROPERTY_ID: u8 = 0x26;
const SUBSCRIPTION_IDENTIFIER_ID: u8 = 0x0B;
const CONTENT_TYPE_ID: u8 = 0x03;

#[derive(Debug, PartialEq, Clone)]
pub struct PublishProperties {
    /// Si vale 0, indica que el payload tiene bytes unspecified -> es equivalente a no enviar un payload format indicator.
    /// Si vale 1 indica que el payload esta encodeado en UTF-8
    pub payload_format_indicator: u8,

    ///Indica el lifetime del application message en segundos.
    ///Si este intervalo expira y el servidor no ha hecho el delivery, se debe eliminar la copia del mensaje para ese subscriptor.
    pub message_expiry_interval: u32,
    pub topic_properties: TopicProperties,
    pub correlation_data: Vec<u8>,
    pub user_property: String,
    pub subscription_identifier: u32,
    pub content_type: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TopicProperties {
    pub topic_alias: u16,
    pub response_topic: String,
}

impl PublishProperties {
    pub fn new(
        payload_format_indicator: u8,
        message_expiry_interval: u32,
        topic_properties: TopicProperties,
        correlation_data: Vec<u8>,
        user_property: String,
        subscription_identifier: u32,
        content_type: String,
    ) -> PublishProperties {
        PublishProperties {
            payload_format_indicator,
            message_expiry_interval,
            topic_properties,
            correlation_data,
            user_property,
            subscription_identifier,
            content_type,
        }
    }

    pub fn write_properties(&self, stream: &mut dyn Write) -> Result<(), ProtocolError> {
        //payload format indicator
        write_u8(stream, &PAYLOAD_FORMAT_INDICATOR_ID)?;
        write_u8(stream, &self.payload_format_indicator)?;

        //message expiry interval
        write_u8(stream, &MESSAGE_EXPIRY_INTERVAL_ID)?;
        write_u32(stream, &self.message_expiry_interval)?;

        //topic alias
        write_u8(stream, &TOPIC_ALIAS_ID)?;
        write_u16(stream, &self.topic_properties.topic_alias)?;

        //response topic
        write_u8(stream, &RESPONSE_TOPIC_ID)?;
        write_string(stream, &self.topic_properties.response_topic)?;

        //correlation data
        write_u8(stream, &CORRELATION_DATA_ID)?;
        write_bin_vec(stream, &self.correlation_data)?;

        //user property
        write_u8(stream, &USER_PROPERTY_ID)?;
        write_string(stream, &self.user_property)?;

        //subscription identifier
        write_u8(stream, &SUBSCRIPTION_IDENTIFIER_ID)?;
        write_u32(stream, &self.subscription_identifier)?;

        //content type
        write_u8(stream, &CONTENT_TYPE_ID)?;
        write_string(stream, &self.content_type)?;

        Ok(())
    }
    #[allow(dead_code)]
    pub fn read_from(stream: &mut dyn Read) -> Result<PublishProperties, Error> {
        //payload format indicator
        let _payload_format_indicator_id = read_u8(stream)?;
        let payload_format_indicator = read_u8(stream)?;

        //message expiry interval
        let _message_expiry_interval_id = read_u8(stream)?;
        let message_expiry_interval = read_u32(stream)?;

        //topic alias
        let _topic_alias_id = read_u8(stream)?;
        let topic_alias = read_u16(stream)?;

        //response topic
        let _response_topic_id = read_u8(stream)?;
        let response_topic = read_string(stream)?;

        let topic_properties = TopicProperties {
            topic_alias,
            response_topic,
        };

        //correlation data
        let _correlation_data_id = read_u8(stream)?;
        let correlation_data = read_bin_vec(stream)?;

        //user property
        let _user_property_id = read_u8(stream)?;
        let user_property = read_string(stream)?;

        //subscription identifier
        let _subscription_identifier_id = read_u8(stream)?;
        let subscription_identifier = read_u32(stream)?;

        //content type
        let _content_type_id = read_u8(stream)?;
        let content_type = read_string(stream)?;

        Ok(PublishProperties {
            payload_format_indicator,
            message_expiry_interval,
            topic_properties,
            correlation_data,
            user_property,
            subscription_identifier,
            content_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_01_publish_properties_ok() {
        let mut buffer = Cursor::new(Vec::new());

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

        match properties.write_properties(&mut buffer) {
            Ok(_) => (),
            Err(err) => panic!("Error writing properties: {:?}", err),
        }
        buffer.set_position(0);

        let publish_properties_read = match PublishProperties::read_from(&mut buffer) {
            Ok(properties) => properties,
            Err(err) => panic!("Error reading properties: {:?}", err),
        };
        assert_eq!(properties, publish_properties_read);
    }
}
