use crate::utils::{reader::*, writer::*};

use std::io::{BufReader, BufWriter, Error, ErrorKind, Read, Write};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct ConnectProperties {
    pub session_expiry_interval: u32,
    pub receive_maximum: u16,
    pub maximum_packet_size: u32,
    pub topic_alias_maximum: u16,
    pub request_response_information: bool,
    pub request_problem_information: bool,
    pub user_properties: Vec<(String, String)>,
    pub authentication_method: String,
    pub authentication_data: Vec<u8>,
}

impl ConnectProperties {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_expiry_interval: u32,
        receive_maximum: u16,
        maximum_packet_size: u32,
        topic_alias_maximum: u16,
        request_response_information: bool,
        request_problem_information: bool,
        user_properties: Vec<(String, String)>,
        authentication_method: String,
        authentication_data: Vec<u8>,
    ) -> ConnectProperties {
        ConnectProperties {
            session_expiry_interval,
            receive_maximum,
            maximum_packet_size,
            topic_alias_maximum,
            request_response_information,
            request_problem_information,
            user_properties,
            authentication_method,
            authentication_data,
        }
    }

    pub fn write_to(&self, stream: &mut dyn Write) -> Result<(), Error> {
        let mut writer = BufWriter::new(stream);

        let session_expiry_interval_id: u8 = 0x11_u8;

        writer.write_all(&[session_expiry_interval_id])?;
        write_u32(&mut writer, &self.session_expiry_interval)?;

        let authentication_method_id: u8 = 0x15_u8;
        writer.write_all(&[authentication_method_id])?;
        write_string(&mut writer, &self.authentication_method)?;

        let authentication_data_id: u8 = 0x16_u8;
        writer.write_all(&[authentication_data_id])?;
        write_bin_vec(&mut writer, &self.authentication_data)?;

        let request_problem_information_id: u8 = 0x17_u8;
        writer.write_all(&[request_problem_information_id])?;
        write_bool(&mut writer, &self.request_problem_information)?;

        let request_response_information_id: u8 = 0x19_u8; // 25

        writer.write_all(&[request_response_information_id])?;
        write_bool(&mut writer, &self.request_response_information)?;

        let receive_maximum_id: u8 = 0x21_u8; // 33
        writer.write_all(&[receive_maximum_id])?;
        write_u16(&mut writer, &self.receive_maximum)?;

        let topic_alias_maximum_id: u8 = 0x22_u8; // 34
        writer.write_all(&[topic_alias_maximum_id])?;
        write_u16(&mut writer, &self.topic_alias_maximum)?;

        let user_properties_id: u8 = 0x26_u8; // 38
        writer.write_all(&[user_properties_id])?;
        write_tuple_vec(&mut writer, &self.user_properties)?;

        let maximum_packet_size_id: u8 = 0x27_u8; // 39
        writer.write_all(&[maximum_packet_size_id])?;
        write_u32(&mut writer, &self.maximum_packet_size)?;

        Ok(())
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<ConnectProperties, Error> {
        let mut reader = BufReader::new(stream);

        let mut session_expiry_interval: Option<u32> = None;
        let mut receive_maximum: Option<u16> = None;
        let mut maximum_packet_size: Option<u32> = None;
        let mut topic_alias_maximum: Option<u16> = None;
        let mut request_response_information: Option<bool> = None;
        let mut request_problem_information: Option<bool> = None;
        let mut user_properties: Option<Vec<(String, String)>> = None;
        let mut authentication_method: Option<String> = None;
        let mut authentication_data: Option<Vec<u8>> = None;
        let mut count = 0;
        while let Ok(property_id) = read_u8(&mut reader) {
            match property_id {
                0x11 => {
                    let value = read_u32(&mut reader)?;
                    session_expiry_interval = Some(value);
                }
                0x15 => {
                    let value = read_string(&mut reader)?;
                    authentication_method = Some(value);
                }
                0x16 => {
                    let value = read_bin_vec(&mut reader)?;
                    authentication_data = Some(value);
                }
                0x17 => {
                    let value = read_bool(&mut reader)?;
                    request_problem_information = Some(value);
                }
                0x19 => {
                    let value = read_bool(&mut reader)?;
                    request_response_information = Some(value);
                }
                0x21 => {
                    let value = read_u16(&mut reader)?;
                    receive_maximum = Some(value);
                }
                0x22 => {
                    let value = read_u16(&mut reader)?;
                    topic_alias_maximum = Some(value);
                }
                0x26 => {
                    let value = read_tuple_vec(&mut reader)?;
                    user_properties = Some(value);
                }
                0x27 => {
                    let value = read_u32(&mut reader)?;
                    maximum_packet_size = Some(value);
                }
                _ => {
                    return Err(Error::new(ErrorKind::InvalidData, "Property ID inválido"));
                }
            }
            count += 1;
            if count == 9 {
                break;
            }
        }

        Ok(ConnectProperties {
            session_expiry_interval: session_expiry_interval.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing session_expiry_interval property",
            ))?,
            receive_maximum: receive_maximum.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing receive_maximum property",
            ))?,
            maximum_packet_size: maximum_packet_size.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing maximum_packet_size property",
            ))?,
            topic_alias_maximum: topic_alias_maximum.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing topic_alias_maximum property",
            ))?,
            request_response_information: request_response_information.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing request_response_information property",
            ))?,
            request_problem_information: request_problem_information.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing request_problem_information property",
            ))?,
            user_properties: user_properties.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing user_properties property",
            ))?,
            authentication_method: authentication_method.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing authentication_method property",
            ))?,
            authentication_data: authentication_data.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing authentication_data property",
            ))?,
        })
    }
}

#[derive(Default)]
pub struct ConnectPropertiesBuilder {
    session_expiry_interval: Option<u32>,
    receive_maximum: Option<u16>,
    maximum_packet_size: Option<u32>,
    topic_alias_maximum: Option<u16>,
    request_response_information: Option<bool>,
    request_problem_information: Option<bool>,
    user_properties: Option<Vec<(String, String)>>,
    authentication_method: Option<String>,
    authentication_data: Option<Vec<u8>>,
}

impl ConnectPropertiesBuilder {
    pub fn create(self) -> Result<ConnectProperties, Error> {
        Ok(ConnectProperties {
            session_expiry_interval: self.session_expiry_interval.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing session_expiry_interval property",
            ))?,
            receive_maximum: self.receive_maximum.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing receive_maximum property",
            ))?,
            maximum_packet_size: self.maximum_packet_size.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing maximum_packet_size property",
            ))?,
            topic_alias_maximum: self.topic_alias_maximum.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing topic_alias_maximum property",
            ))?,
            request_response_information: self.request_response_information.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing request_response_information property",
            ))?,
            request_problem_information: self.request_problem_information.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing request_problem_information property",
            ))?,
            user_properties: self.user_properties.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing user_properties property",
            ))?,
            authentication_method: self.authentication_method.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing authentication_method property",
            ))?,
            authentication_data: self.authentication_data.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing authentication_data property",
            ))?,
        })
    }

    pub fn session_expiry_interval(mut self, value: u32) -> Self {
        self.session_expiry_interval = Some(value);
        self
    }

    pub fn receive_maximum(mut self, value: u16) -> Self {
        self.receive_maximum = Some(value);
        self
    }

    pub fn maximum_packet_size(mut self, value: u32) -> Self {
        self.maximum_packet_size = Some(value);
        self
    }

    pub fn topic_alias_maximum(mut self, value: u16) -> Self {
        self.topic_alias_maximum = Some(value);
        self
    }

    pub fn request_response_information(mut self, value: bool) -> Self {
        self.request_response_information = Some(value);
        self
    }

    pub fn request_problem_information(mut self, value: bool) -> Self {
        self.request_problem_information = Some(value);
        self
    }

    pub fn user_properties(mut self, value: Vec<(String, String)>) -> Self {
        self.user_properties = Some(value);
        self
    }

    pub fn authentication_method(mut self, value: String) -> Self {
        self.authentication_method = Some(value);
        self
    }

    pub fn authentication_data(mut self, value: Vec<u8>) -> Self {
        self.authentication_data = Some(value);
        self
    }
}
#[allow(dead_code)]

fn read_json_to_connect_properties(json_data: &str) -> Result<ConnectProperties, Error> {
    let connect_properties: ConnectProperties = serde_json::from_str(json_data)?;
    Ok(connect_properties)
}
#[cfg(test)]
mod tests {
    use core::panic;
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_read_json_to_connect_properties() {
        let json_data = r#"{
            "session_expiry_interval": 30,
            "receive_maximum": 1,
            "maximum_packet_size": 20,
            "topic_alias_maximum": 20,
            "request_response_information": true,
            "request_problem_information": true,
            "user_properties": [
                [
                    "hola",
                    "chau"
                ]
            ],
            "authentication_method": "password-based",
            "authentication_data": [
                1,
                2,
                3
            ]
        }"#;
        let connect_properties = read_json_to_connect_properties(json_data).unwrap();
        let expected_connect_properties = ConnectProperties::new(
            30,
            1,
            20,
            20,
            true,
            true,
            vec![("hola".to_string(), "chau".to_string())],
            "password-based".to_string(),
            vec![1, 2, 3],
        );
        assert_eq!(connect_properties, expected_connect_properties);
    }

    #[test]
    fn test_01_connect_properties_ok() {
        let mut buffer = Cursor::new(Vec::new());
        let connect_properties = ConnectProperties {
            session_expiry_interval: 1,
            receive_maximum: 2,
            maximum_packet_size: 10,
            topic_alias_maximum: 99,
            request_response_information: true,
            request_problem_information: false,
            user_properties: vec![
                ("Hola".to_string(), "Mundo".to_string()),
                ("Chau".to_string(), "Mundo".to_string()),
            ],
            authentication_method: "test".to_string(),
            authentication_data: vec![1_u8, 2_u8, 3_u8, 4_u8, 5_u8],
        };

        match connect_properties.write_to(&mut buffer) {
            Ok(_) => {}
            Err(e) => {
                println!("Error: {:?}", e);
                panic!();
            }
        }
        buffer.set_position(0);

        let connect_properties_read = match ConnectProperties::read_from(&mut buffer) {
            Ok(properties) => properties,
            Err(e) => {
                println!("Error: {:?}", e);

                panic!();
            }
        };
        assert_eq!(connect_properties, connect_properties_read);
    }

    #[test]
    fn connect_properties_builder() {
        let connect_properties = ConnectPropertiesBuilder::default()
            .session_expiry_interval(1)
            .receive_maximum(2)
            .maximum_packet_size(10)
            .topic_alias_maximum(99)
            .request_response_information(true)
            .request_problem_information(false)
            .user_properties(vec![
                ("Hola".to_string(), "Mundo".to_string()),
                ("Chau".to_string(), "Mundo".to_string()),
            ])
            .authentication_method("test".to_string())
            .authentication_data(vec![1_u8, 2_u8, 3_u8, 4_u8, 5_u8])
            .create()
            .unwrap();

        assert_eq!(connect_properties.session_expiry_interval, 1);
        assert_eq!(connect_properties.receive_maximum, 2);
        assert_eq!(connect_properties.maximum_packet_size, 10);
        assert_eq!(connect_properties.topic_alias_maximum, 99);
        assert!(connect_properties.request_response_information);
        assert!(!connect_properties.request_problem_information);
        assert_eq!(
            connect_properties.user_properties,
            vec![
                ("Hola".to_string(), "Mundo".to_string()),
                ("Chau".to_string(), "Mundo".to_string()),
            ]
        );
        assert_eq!(connect_properties.authentication_method, "test");
        assert_eq!(
            connect_properties.authentication_data,
            vec![1_u8, 2_u8, 3_u8, 4_u8, 5_u8]
        );
    }
}
