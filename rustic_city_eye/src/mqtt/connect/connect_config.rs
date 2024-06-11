use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use crate::mqtt::protocol_error::ProtocolError;
use crate::mqtt::{
    connect::connect_properties::ConnectProperties, will_properties::WillProperties,
};

use crate::mqtt::{client_message::ClientMessage, messages_config::MessagesConfig};
use serde::{Deserialize, Serialize};

/// Cada vez que el usuario de la API de Client intenta enviar un packet
/// del tipo Connect, debe enviar un ConnectConfig, que contendra
/// la informacion con la que se va a armar el packet de Connect.
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct ConnectConfig {
    pub(crate) clean_start: bool,
    pub(crate) last_will_flag: bool,
    pub(crate) last_will_qos: u8,
    pub(crate) last_will_retain: bool,
    pub(crate) keep_alive: u16,
    pub(crate) properties: ConnectProperties,

    /// Payload
    /// Ayuda a que el servidor identifique al cliente. Siempre debe ser
    /// el primer campo del payload del packet Connect.
    pub(crate) client_id: String,

    pub(crate) will_properties: WillProperties,
    pub(crate) last_will_topic: String,
    pub(crate) last_will_message: String,
    pub(crate) username: String,
    pub(crate) password: String,
}

impl MessagesConfig for ConnectConfig {
    /// Hereda del trait de MessagesConfig, por lo que sabe
    /// crear un ClientMessage -> en este caso devolvera
    /// siempre un mensaje del tipo Connect.
    fn parse_message(&self, _packet_id: u16) -> ClientMessage {
        ClientMessage::Connect {
            clean_start: self.clean_start,
            last_will_flag: self.last_will_flag,
            last_will_qos: self.last_will_qos,
            last_will_retain: self.last_will_retain,
            keep_alive: self.keep_alive,
            properties: self.properties.clone(),
            client_id: self.client_id.clone(),
            will_properties: self.will_properties.clone(),
            last_will_topic: self.last_will_topic.clone(),
            last_will_message: self.last_will_message.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
        }
    }
}

impl ConnectConfig {
    #[allow(clippy::too_many_arguments)]
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
            will_properties,
            last_will_topic,
            last_will_message,
            username,
            password,
        }
    }

    /// Abre un archivo de configuracion con propiedades y guarda sus lecturas.
    pub fn read_connect_config(file_path: &str) -> Result<ConnectConfig, ProtocolError> {
        let config_file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => return Err(ProtocolError::ReadingConfigFileError),
        };

        let reader: BufReader<File> = BufReader::new(config_file);
        let config = match serde_json::from_reader(reader) {
            Ok(c) => c,
            Err(_) => return Err(ProtocolError::ReadingConfigFileError),
        };

        println!("config {:?}", config);

        Ok(config)
    }
}
#[allow(dead_code)]

fn read_json_to_connect_config(json_data: &str) -> Result<ConnectConfig, Box<dyn Error>> {
    let connect_config: ConnectConfig = serde_json::from_str(json_data)?;
    Ok(connect_config)
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
        let connect_config = read_json_to_connect_config(json_data).unwrap();
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
                clean_start: true,
                last_will_flag: true,
                last_will_qos: 1,
                last_will_retain: true,
                keep_alive: 35,
                properties: connect_properties,
                client_id: "juancito".to_string(),
                will_properties,
                last_will_topic: "camera system".to_string(),
                last_will_message: "soy el monitoring y me desconecte".to_string(),
                username: "a".to_string(),
                password: "a".to_string(),
            }
        );
    }
}
