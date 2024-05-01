use std::io::{Error, Read, Write};

use crate::mqtt::writer::*;
use crate::mqtt::reader::*;

//PROPERTIES IDs
const PAYLOAD_FORMAT_INDICATOR_ID: u8 = 0x01;
const MESSAGE_EXPIRY_INTERVAL_ID: u8 = 0x02;
const TOPIC_ALIAS_ID: u8 = 0x23;
const RESPONSE_TOPIC_ID: u8 = 0x08;
const CORRELATION_DATA_ID: u8 = 0x09;
const USER_PROPERTY_ID: u8 = 0x26;
const SUBSCRIPTION_IDENTIFIER_ID: u8 = 0x0B;
const CONTENT_TYPE_ID: u8 = 0x03;


#[derive(Debug)]
pub struct PublishProperties {
    payload_format_indicator_id: u8,
    payload_format_indicator: u8,
    message_expiry_interval_id: u8,
    message_expiry_interval: u32,
    topic_alias_id: u8,
    topic_alias: u16,
    response_topic_id: u8,
    response_topic: String,
    correlation_data_id: u8,
    correlation_data: Vec<u8>,
    user_property_id: u8,
    user_property: String,
    subscription_identifier_id: u8,
    subscription_identifier: u32,
    content_type_id: u8,
    content_type: String
}

impl PublishProperties {
    pub fn new() -> PublishProperties {
        PublishProperties { 
            payload_format_indicator_id: PAYLOAD_FORMAT_INDICATOR_ID,
            payload_format_indicator: 1,
            message_expiry_interval_id: MESSAGE_EXPIRY_INTERVAL_ID,
            message_expiry_interval: 10,
            topic_alias_id: TOPIC_ALIAS_ID,
            topic_alias: 10,
            response_topic_id: RESPONSE_TOPIC_ID,
            response_topic: "String".to_string(),
            correlation_data_id: CORRELATION_DATA_ID,
            correlation_data: [1, 2, 3].to_vec(),
            user_property_id: USER_PROPERTY_ID,
            user_property: "String".to_string(),
            subscription_identifier_id: SUBSCRIPTION_IDENTIFIER_ID,
            subscription_identifier: 1,
            content_type_id: CONTENT_TYPE_ID,
            content_type: "String".to_string()
        }
    }

    pub fn write_properties(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        //payload format indicator
        write_u8(stream, &self.payload_format_indicator_id)?;
        write_u8(stream, &self.payload_format_indicator)?;

        //message expiry interval
        write_u8(stream, &self.message_expiry_interval_id)?;
        write_u32(stream, &self.message_expiry_interval)?;

        //topic alias
        write_u8(stream, &self.topic_alias_id)?;
        write_u16(stream, &self.topic_alias)?;

        //response topic
        write_u8(stream, &self.response_topic_id)?;
        write_string(stream, &self.response_topic)?;

        //correlation data
        // write_u8(stream, &self.correlation_data_id)?;

        // for byte in &self.correlation_data {
        //     write_u8(stream, byte)?;
        // }

        //user property
        write_u8(stream, &self.user_property_id)?;
        write_string(stream, &self.user_property)?;
        
        //subscription identifier
        write_u8(stream, &self.subscription_identifier_id)?;
        write_u32(stream, &self.subscription_identifier)?;

        //content type
        write_u8(stream, &self.content_type_id)?;
        write_string(stream, &self.content_type)?;


        Ok(())
    }

    pub fn read_properties(&self, stream: &mut dyn Read) -> Result<PublishProperties, Error> {
        //payload format indicator
        let payload_format_indicator_id = read_u8(stream)?;
        let payload_format_indicator = read_u8(stream)?;

        //message expiry interval
        let message_expiry_interval_id = read_u8(stream)?;
        let message_expiry_interval = read_u32(stream)?;

        //topic alias
        let topic_alias_id = read_u8(stream)?;
        let topic_alias = read_u16(stream)?;

        //response topic
        let response_topic_id = read_u8(stream)?;
        let response_topic = read_string(stream)?;

        //correlation data
        //let correlation_data_id = read_u8(stream)?;

        // let correlation_data_len = read_u16(stream)?;
        // let correlation_data: Vec<u8> = vec![0; correlation_data_len as usize];

        //user property
        let user_property_id = read_u8(stream)?;
        let user_property = read_string(stream)?;
        
        //subscription identifier
        let subscription_identifier_id = read_u8(stream)?;
        let subscription_identifier = read_u32(stream)?;

        //content type
        let content_type_id = read_u8(stream)?;
        let content_type = read_string(stream)?;

        Ok(PublishProperties { 
            payload_format_indicator_id,
            payload_format_indicator,
            message_expiry_interval_id,
            message_expiry_interval,
            topic_alias_id,
            topic_alias,
            response_topic_id,
            response_topic,
            correlation_data_id: 1,
            correlation_data: [1, 1, 1].to_vec(),
            user_property_id,
            user_property,
            subscription_identifier_id,
            subscription_identifier,
            content_type_id,
            content_type
        })
    }
}