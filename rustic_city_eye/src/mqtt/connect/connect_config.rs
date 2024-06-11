use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::Error;
use std::io::{BufReader, Read, Write};

use crate::mqtt::protocol_error::ProtocolError;
use crate::mqtt::{client_message::ClientMessage, messages_config::MessagesConfig};
use crate::mqtt::{
    connect::connect_properties::ConnectProperties, connect::will_properties::WillProperties,
};
use crate::utils::reader::*;
use crate::utils::writer::*;

/// Cada vez que el usuario de la API de Client intenta enviar un packet
/// del tipo Connect, debe enviar un ConnectConfig, que contendra
/// la informacion con la que se va a armar el packet de Connect.
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct ConnectConfig {
    clean_start: bool,
    last_will_flag: bool,
    last_will_qos: u8,
    last_will_retain: bool,
    keep_alive: u16,
    properties: ConnectProperties,

    /// Connect Payload
    /// Ayuda a que el servidor identifique al cliente. Siempre debe ser
    /// el primer campo del payload del packet Connect.
    pub(crate) client_id: String,

    will_properties: Option<WillProperties>,
    last_will_topic: Option<String>,
    last_will_message: Option<String>,
    username: Option<String>,
    password: Option<Vec<u8>>,
}

impl MessagesConfig for ConnectConfig {
    /// Hereda del trait de MessagesConfig, por lo que sabe
    /// crear un ClientMessage -> en este caso devolvera
    /// siempre un mensaje del tipo Connect.
    fn parse_message(&self, _packet_id: u16) -> ClientMessage {
        ClientMessage::Connect {
            connect_config: self.clone(),
        }
    }
}

impl ConnectConfig {
    pub fn new(
        clean_start: bool,
        last_will_flag: bool,
        last_will_qos: u8,
        last_will_retain: bool,
        keep_alive: u16,
        properties: ConnectProperties,
        client_id: String,
        will_properties: WillProperties,
        last_will_topic: String,
        last_will_message: String,
        username: String,
        password: String,
    ) -> ConnectConfig {
        ConnectConfig {
            clean_start,
            last_will_flag,
            last_will_qos,
            last_will_retain,
            keep_alive,
            properties,
            client_id,
            will_properties: Some(will_properties),
            last_will_topic: Some(last_will_topic),
            last_will_message: Some(last_will_message),
            username: Some(username),
            password: Some(password.into_bytes()),
        }
    }
    /// Abre un archivo de configuracion con propiedades y guarda sus lecturas.
    pub fn read_connect_config(file_path: &str) -> Result<ConnectConfig, ProtocolError> {
        let config_file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => return Err(ProtocolError::ReadingConfigFileError),
        };

        let reader: BufReader<File> = BufReader::new(config_file);
        let config: ConnectConfig = match serde_json::from_reader(reader) {
            Ok(c) => c,
            Err(_) => return Err(ProtocolError::ReadingConfigFileError),
        };

        config.check_will_properties()?;

        Ok(config)
    }

    fn check_will_properties(&self) -> Result<(), ProtocolError> {
        if let (Some(_), Some(_), Some(_)) = (
            &self.will_properties,
            &self.last_will_topic,
            &self.last_will_message,
        ) {
            if self.last_will_qos > 1 {
                //si es una QoS no soportada...
                return Err(ProtocolError::InvalidQOS);
            }
            return Ok(());
        }

        Err(ProtocolError::MissingWillMessageProperties)
    }

    pub fn write_to(&self, writer: &mut dyn Write) -> Result<(), ProtocolError> {
        //connection flags
        let mut connect_flags: u8 = 0x00;
        if self.clean_start {
            connect_flags |= 1 << 1; //set bit 1 to 1
        }

        if self.last_will_flag {
            connect_flags |= 1 << 2;
        }
        if self.last_will_qos > 1 {
            return Err(ProtocolError::InvalidQOS);
        }
        connect_flags |= (self.last_will_qos & 0b11) << 3;

        if self.last_will_retain {
            connect_flags |= 1 << 5;
        }

        if let Some(password) = self.password.as_ref() {
            if !password.is_empty() {
                connect_flags |= 1 << 6;
            }
        }

        if let Some(username) = self.username.as_ref() {
            if !username.is_empty() {
                connect_flags |= 1 << 7;
            }
        }

        let _ = writer
            .write_all(&[connect_flags])
            .map_err(|_e| ProtocolError::WriteError);

        //keep alive
        write_u16(writer, &self.keep_alive)?;
        write_string(writer, &self.client_id)?;

        if let (Some(will_properties), Some(last_will_topic), Some(last_will_message)) = (
            self.will_properties.clone(),
            self.last_will_topic.clone(),
            self.last_will_message.clone(),
        ) {
            if self.last_will_flag {
                will_properties.write_to(writer)?;
                write_string(writer, &last_will_topic)?;
                write_string(writer, &last_will_message)?;
            }
        }

        if let Some(username) = self.username.as_ref() {
            if !username.is_empty() {
                write_string(writer, username)?;
            }
        }

        if let Some(password) = self.password.as_ref() {
            if !password.is_empty() {
                write_bin_vec(writer, password)?;
            }
        }

        self.properties.write_to(writer)?;

        Ok(())
    }

    pub fn read_from(stream: &mut impl Read) -> Result<ConnectConfig, Error> {
        //connect flags
        let connect_flags = read_u8(stream)?;
        let clean_start = (connect_flags & (1 << 1)) != 0;
        let last_will_flag = (connect_flags & (1 << 2)) != 0;
        let last_will_qos = (connect_flags >> 3) & 0b11;
        let last_will_retain = (connect_flags & (1 << 5)) != 0;

        //keep alive
        let keep_alive = read_u16(stream)?;
        //payload
        //client ID
        let client_id = read_string(stream)?;

        let mut last_will_topic = String::new();
        let mut will_message = String::new();

        let will_properties = WillProperties::read_from(stream)?;
        if last_will_flag {
            last_will_topic = read_string(stream)?;
            will_message = read_string(stream)?;
        }

        let hay_user = (connect_flags & (1 << 7)) != 0;
        let mut user = String::new();
        if hay_user {
            user = read_string(stream)?;
        }

        let hay_pass = (connect_flags & (1 << 6)) != 0;
        let mut pass = Vec::new();
        if hay_pass {
            pass = read_bin_vec(stream)?;
        }

        //properties
        let properties: ConnectProperties = ConnectProperties::read_from(stream)?;

        Ok(ConnectConfig {
            clean_start,
            last_will_flag,
            last_will_qos,
            last_will_retain,
            keep_alive,
            properties,
            client_id,
            will_properties: Some(will_properties),
            last_will_topic: Some(last_will_topic),
            last_will_message: Some(will_message),
            username: Some(user),
            password: Some(pass),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_config_creation_cases() {
        let config_ok = ConnectConfig::read_connect_config("./src/monitoring/connect_config.json");

        let config_err = ConnectConfig::read_connect_config("este/es/un/path/feo");

        assert!(config_ok.is_ok());
        assert!(config_err.is_err());
    }

    #[test]
    fn test_02_config_without_last_will_msg_throws_err() {
        let config_err = ConnectConfig::read_connect_config(
            "./tests/connect_config_test/config_without_will_msg.json",
        );

        assert!(config_err.is_err());
    }

    #[test]
    fn test_03_config_with_lat_will_invalid_qos_err() {
        let config_err = ConnectConfig::read_connect_config(
            "./tests/connect_config_test/connect_config_invalid_qos.json",
        );

        assert!(config_err.is_err());
    }

    #[test]
    fn test_read_json_connect_config() {
        let json_data = r#"{
            "clean_start": true,
            "last_will_flag": true,
            "last_will_qos": 1,
            "last_will_retain": true,
            "keep_alive": 35,
            "properties": {
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
            },
            "client_id": "juancito",
            "will_properties": {
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
            },
            "last_will_topic": "camera system",
            "last_will_message": "soy el monitoring y me desconecte",
            "username": "a",
            "password": "a"
        }"#;
        let connect_config = ConnectConfig::read_connect_config(json_data).unwrap();
        let expected_connect_config = ConnectConfig::new(
            true,
            true,
            1,
            true,
            35,
            ConnectProperties::new(
                30,
                1,
                20,
                20,
                true,
                true,
                vec![("hola".to_string(), "chau".to_string())],
                "password-based".to_string(),
                vec![1, 2, 3],
            ),
            "juancito".to_string(),
            WillProperties::new(
                1,
                1,
                1,
                "a".to_string(),
                "a".to_string(),
                [1, 2, 3].to_vec(),
                vec![("a".to_string(), "a".to_string())],
            ),
            "camera system".to_string(),
            "soy el monitoring y me desconecte".to_string(),
            "a".to_string(),
            "a".to_string(),
        );
        assert_eq!(connect_config, expected_connect_config);
    }

    #[test]
    fn test_parse_message() {
        let connect_properties = ConnectProperties::new(
            30,
            1,
            20,
            20,
            true,
            true,
            vec![("hola".to_string(), "chau".to_string())],
            "auth".to_string(),
            vec![1, 2, 3],
        );

        let will_properties = WillProperties::new(
            1,
            1,
            1,
            "a".to_string(),
            "a".to_string(),
            [1, 2, 3].to_vec(),
            vec![("a".to_string(), "a".to_string())],
        );

        let connect_config = ConnectConfig::new(
            true,
            true,
            1,
            true,
            35,
            connect_properties.clone(),
            "juancito".to_string(),
            will_properties.clone(),
            "camera system".to_string(),
            "soy el monitoring y me desconecte".to_string(),
            "a".to_string(),
            "a".to_string(),
        );

        let connect_message = connect_config.parse_message(1);

        assert_eq!(
            connect_message,
            ClientMessage::Connect {
                connect_config: connect_config.clone()
            }
        );
    }
}
