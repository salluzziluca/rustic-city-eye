use std::io::{Error, Read, Write};

use crate::mqtt::reader::*;
use crate::mqtt::writer::*;

//PROPERTIES IDs
const PAYLOAD_FORMAT_INDICATOR_ID: u8 = 0x01;
const MESSAGE_EXPIRY_INTERVAL_ID: u8 = 0x02;
const TOPIC_ALIAS_ID: u8 = 0x23;
const RESPONSE_TOPIC_ID: u8 = 0x08;
const USER_PROPERTY_ID: u8 = 0x26;
const SUBSCRIPTION_IDENTIFIER_ID: u8 = 0x0B;
const CONTENT_TYPE_ID: u8 = 0x03;

#[derive(Debug, PartialEq)]
pub struct PublishProperties {
    payload_format_indicator: u8,
    message_expiry_interval: u32,
    topic_alias: u16,
    response_topic: String,
    correlation_data: Vec<u8>,
    user_property: String,
    subscription_identifier: u32,
    content_type: String,
}

impl PublishProperties {
    pub fn new(
        payload_format_indicator: u8,
        message_expiry_interval: u32,
        topic_alias: u16,
        response_topic: String,
        correlation_data: Vec<u8>,
        user_property: String,
        subscription_identifier: u32,
        content_type: String,
    ) -> PublishProperties {
        PublishProperties {
            payload_format_indicator,
            message_expiry_interval,
            topic_alias,
            response_topic,
            correlation_data,
            user_property,
            subscription_identifier,
            content_type,
        }
    }

    pub fn write_properties(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        //payload format indicator
        write_u8(stream, &PAYLOAD_FORMAT_INDICATOR_ID)?;
        write_u8(stream, &self.payload_format_indicator)?;

        //message expiry interval
        write_u8(stream, &MESSAGE_EXPIRY_INTERVAL_ID)?;
        write_u32(stream, &self.message_expiry_interval)?;

        //topic alias
        write_u8(stream, &TOPIC_ALIAS_ID)?;
        write_u16(stream, &self.topic_alias)?;

        //response topic
        write_u8(stream, &RESPONSE_TOPIC_ID)?;
        write_string(stream, &self.response_topic)?;

        //correlation data
        // write_u8(stream, &self.correlation_data_id)?;

        // for byte in &self.correlation_data {
        //     write_u8(stream, byte)?;
        // }

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
    pub fn read_properties(&self, stream: &mut dyn Read) -> Result<PublishProperties, Error> {
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

        //correlation data
        //let correlation_data_id = read_u8(stream)?;

        // let correlation_data_len = read_u16(stream)?;
        // let correlation_data: Vec<u8> = vec![0; correlation_data_len as usize];

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
            topic_alias,
            response_topic,
            correlation_data: [1, 1, 1].to_vec(),
            user_property,
            subscription_identifier,
            content_type,
        })
    }
}
