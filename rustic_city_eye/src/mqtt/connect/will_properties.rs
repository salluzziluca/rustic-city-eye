use serde::{Deserialize, Serialize};

use std::io::BufWriter;
use std::io::Error;
use std::io::Read;
use std::io::Write;

use crate::mqtt::protocol_error::ProtocolError;
use crate::mqtt::publish::publish_properties::PublishProperties;
use crate::mqtt::publish::publish_properties::TopicProperties;
use crate::utils::{reader::*, writer::*};

const WILL_DELAY_INTERVAL_ID: u8 = 0x18;
const PAYLOAD_FORMAT_INDICATOR_ID: u8 = 0x01;
const MESSAGE_EXPIRY_INTERVAL_ID: u8 = 0x02;
const CONTENT_TYPE_ID: u8 = 0x03;
const RESPONSE_TOPIC_ID: u8 = 0x08;
const CORRELATION_DATA_ID: u8 = 0x09;
const USER_PROPERTIES_ID: u8 = 0x26;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
/// last_will_delay_interval especifica el tiempo en segundos que el broker debe esperar antes de publicar el will message.
///
/// payload_format_indicator indica si el payload esta encodado en utf-8 o no.
/// message_expiry_interval especifica el tiempo en segundos que el broker debe esperar antes de descartar el will message.
/// content_type especifica el tipo de contenido del will message. (ej json, plain text)
/// el response_topic es el el topic name que debera usar el mensaje de respuesta. Si hay response_topic, el will message se considera un request message.
///
/// `The Correlation Data is used by the sender of the Request Message to identify which request the Response Message is for when it is received`
///
/// user_property especifica una propiedad del usuario que se envia en el mensaje, se pueden enviar 0, 1 o más propiedades.
pub struct WillProperties {
    last_will_delay_interval: u32,
    payload_format_indicator: u8,
    message_expiry_interval: u16,
    content_type: String,
    response_topic: String,
    correlation_data: Vec<u8>,
    user_properties: Vec<(String, String)>,
}

impl WillProperties {
    pub fn new(
        last_will_delay_interval: u32,
        payload_format_indicator: u8,
        message_expiry_interval: u16,
        content_type: String,
        response_topic: String,
        correlation_data: Vec<u8>,
        user_properties: Vec<(String, String)>,
    ) -> WillProperties {
        WillProperties {
            last_will_delay_interval,
            payload_format_indicator,
            message_expiry_interval,
            content_type,
            response_topic,
            correlation_data,
            user_properties,
        }
    }
    pub fn write_to(&self, stream: &mut dyn Write) -> Result<(), ProtocolError> {
        let mut writer = BufWriter::new(stream);
        //will properties
        write_u8(&mut writer, &WILL_DELAY_INTERVAL_ID)?;

        write_u32(&mut writer, &self.last_will_delay_interval)?;

        let payload_format_indicator = 0x01_u8;
        //siempre 1 porque los strings en rust siempre son utf-8
        write_u8(&mut writer, &PAYLOAD_FORMAT_INDICATOR_ID)?;
        write_u8(&mut writer, &payload_format_indicator)?;

        write_u8(&mut writer, &MESSAGE_EXPIRY_INTERVAL_ID)?;

        write_u16(&mut writer, &self.message_expiry_interval)?;

        write_u8(&mut writer, &CONTENT_TYPE_ID)?;
        write_string(&mut writer, &self.content_type)?;

        write_u8(&mut writer, &RESPONSE_TOPIC_ID)?;
        write_string(&mut writer, &self.response_topic)?;

        write_u8(&mut writer, &CORRELATION_DATA_ID)?;
        let correlation_data_length = self.correlation_data.len() as u16;
        write_u16(&mut writer, &correlation_data_length)?;
        for byte in &self.correlation_data {
            write_u8(&mut writer, byte)?;
        }
        //user property

        write_u8(&mut writer, &USER_PROPERTIES_ID)?;
        let user_properties_length = self.user_properties.len() as u16;
        write_u16(&mut writer, &user_properties_length)?;
        for (key, value) in &self.user_properties {
            write_string(&mut writer, key)?;
            write_string(&mut writer, value)?;
        }

        Ok(())
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<WillProperties, Error> {
        //will properties
        let will_delay_interval_id = read_u8(stream)?;
        if will_delay_interval_id != WILL_DELAY_INTERVAL_ID {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid will delay interval id",
            ));
        }
        let will_delay_interval = read_u32(stream)?;

        let payload_format_indicator_id = read_u8(stream)?;
        if payload_format_indicator_id != PAYLOAD_FORMAT_INDICATOR_ID {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid payload format indicator id",
            ));
        }
        let payload_format_indicator = read_u8(stream)?;

        let message_expiry_interval_id = read_u8(stream)?;
        if message_expiry_interval_id != MESSAGE_EXPIRY_INTERVAL_ID {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid message expiry interval id",
            ));
        }
        let message_expiry_interval = read_u16(stream)?;
        let content_type_id = read_u8(stream)?;
        if content_type_id != CONTENT_TYPE_ID {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid content type id",
            ));
        }
        let content_type = read_string(stream)?;

        let response_topic_id = read_u8(stream)?;
        if response_topic_id != RESPONSE_TOPIC_ID {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid response topic id",
            ));
        }
        let response_topic = read_string(stream)?;

        let correlation_data_id = read_u8(stream)?;
        if correlation_data_id != CORRELATION_DATA_ID {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid correlation data id",
            ));
        }
        let correlation_data_length = read_u16(stream)?;
        let mut correlation_data = vec![0; correlation_data_length as usize];
        stream.read_exact(&mut correlation_data)?;

        //user property
        let user_properties_id = read_u8(stream)?;
        if user_properties_id != USER_PROPERTIES_ID {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid user properties id",
            ));
        }
        let mut user_properties = Vec::new();
        let user_properties_lenght = read_u16(stream)?;
        for _ in 0..user_properties_lenght {
            let user_properties_key = read_string(stream)?;
            let user_properties_value = read_string(stream)?;
            user_properties.push((user_properties_key, user_properties_value));
        }

        Ok(WillProperties {
            last_will_delay_interval: will_delay_interval,
            payload_format_indicator,
            message_expiry_interval,
            content_type,
            response_topic,
            correlation_data,
            user_properties,
        })
    }

    pub fn get_last_will_delay_interval(&self) -> u32 {
        self.last_will_delay_interval
    }
    /// Convierte las will properties en publish properties
    pub fn to_publish_properties(&self) -> PublishProperties {
        let topic_properties = TopicProperties {
            topic_alias: 10,
            response_topic: "String".to_string(),
        };
        let first_property = self.user_properties.first().unwrap();
        PublishProperties::new(
            self.payload_format_indicator,
            self.message_expiry_interval as u32,
            topic_properties,
            self.correlation_data.clone(),
            first_property.0.clone(),
            3,
            self.response_topic.clone(),
        )
    }
}
#[allow(dead_code)]
fn read_json_to_will_properties(json_data: &str) -> Result<WillProperties, Error> {
    let will_properties: WillProperties = serde_json::from_str(json_data)?;
    Ok(will_properties)
}
#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_read_json_to_will_properties() {
        let json_data = r#"{
            "last_will_delay_interval": 1,
            "payload_format_indicator": 1,
            "message_expiry_interval": 1,
            "content_type": "a",
            "response_topic": "a",
            "correlation_data": [
                1,
                2,
                3
            ],
            "user_properties": [
                [
                    "a",
                    "a"
                ]
            ]
        }"#;
        let will_properties = read_json_to_will_properties(json_data).unwrap();
        let expected_will_properties = WillProperties::new(
            1,
            1,
            1,
            "a".to_string(),
            "a".to_string(),
            vec![1, 2, 3],
            vec![("a".to_string(), "a".to_string())],
        );
        assert_eq!(will_properties, expected_will_properties);
    }
    #[test]
    fn test_01_will_properties_ok() {
        let will_properties = WillProperties::new(
            120,
            1,
            30,
            "plain".to_string(),
            "topic".to_string(),
            vec![1, 2, 3, 4, 5],
            vec![("propiedad".to_string(), "valor".to_string())],
        );
        let mut cursor = Cursor::new(Vec::<u8>::new());
        match will_properties.write_to(&mut cursor) {
            Ok(_) => (),
            Err(err) => panic!("Error writing will properties: {:?}", err),
        };
        cursor.set_position(0);

        let read_will_properties = match WillProperties::read_from(&mut cursor) {
            Ok(properties) => properties,
            Err(err) => panic!("Error reading will properties: {:?}", err),
        };

        assert_eq!(will_properties, read_will_properties);
    }

    #[test]
    fn test_to_publish_properties() {
        let will_properties = WillProperties::new(
            120,
            1,
            30,
            "plain".to_string(),
            "topic".to_string(),
            vec![1, 2, 3, 4, 5],
            vec![("propiedad".to_string(), "valor".to_string())],
        );
        let publish_properties = will_properties.to_publish_properties();
        let expected_publish_properties = PublishProperties::new(
            1,
            30,
            TopicProperties {
                topic_alias: 10,
                response_topic: "String".to_string(),
            },
            vec![1, 2, 3, 4, 5],
            "propiedad".to_string(),
            3,
            "topic".to_string(),
        );
        assert_eq!(publish_properties, expected_publish_properties);
    }
}
