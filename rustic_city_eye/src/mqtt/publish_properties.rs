use std::net::TcpStream;

use super::client_message::write_u8;

//PROPERTIES IDs
const PAYLOAD_FORMAT_INDICATOR_ID: u8 = 0x01;
const MESSAGE_EXPIRY_INTERVAL_ID: u8 = 0x02;
const TOPIC_ALIAS_ID: u8 = 0x23;
const RESPONSE_TOPIC_ID: u8 = 0x08;
const CORRELATION_DATA_ID: u8 = 0x09;
const USER_PROPERTY_ID: u8 = 0x26;
const SUBSCRIPTION_IDENTIFIER_ID: u8 = 0x0B;
const CONTENT_TYPE_ID: u8 = 0x03;




pub fn read_string(stream: &mut dyn Read)-> Result<String, Error>{
    let string_length = read_u16(stream)?;
    let mut string_buf = vec![0; string_length as usize];
    stream.read_exact(&mut string_buf)?;

    let protocol_name =
        std::str::from_utf8(&string_buf).expect("Error al leer protocol_name");
    Ok(protocol_name.to_string())
}

pub fn read_u8(stream: &mut dyn Read) -> Result<u8, Error> {
    let mut buf = [0u8; 1];
    stream.read_exact(&mut buf)?;
    Ok(u8::from_be_bytes(buf))
}

pub fn read_u16(stream: &mut dyn Read) -> Result<u16, Error> {
    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

pub fn read_u32(stream: &mut dyn Read) -> Result<u32, Error> {
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

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
    correlation_data: bool,
    user_property_id: u8,
    user_property: String,
    subscription_identifier_id: u8,
    subscription_identifier: usize,
    content_type_id: u8,
    content_type: String
}

impl PublishProperties {
    pub fn new() -> PublishProperties {
        PublishProperties { 
            payload_format_indicator_id: PAYLOAD_FORMAT_INDICATOR_ID,
            payload_format_indicator: 2,
            message_expiry_interval_id: MESSAGE_EXPIRY_INTERVAL_ID,
            message_expiry_interval: 10,
            topic_alias_id: TOPIC_ALIAS_ID,
            topic_alias: 10,
            response_topic_id: RESPONSE_TOPIC_ID,
            response_topic: "String".to_string(),
            correlation_data_id: CORRELATION_DATA_ID,
            correlation_data: true,
            user_property_id: USER_PROPERTY_ID,
            user_property: "String".to_string(),
            subscription_identifier_id: SUBSCRIPTION_IDENTIFIER_ID,
            subscription_identifier: 1,
            content_type_id: CONTENT_TYPE_ID,
            content_type: "String".to_string()
        }
    }

    pub fn write_properties(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        write_u8(stream, &self.payload_format_indicator_id)?;
        write_u8(stream, &self.payload_format_indicator)?;
    }
}