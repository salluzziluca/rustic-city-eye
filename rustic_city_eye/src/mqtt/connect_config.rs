use crate::mqtt::{connect_properties::ConnectProperties, will_properties::WillProperties};

use super::{client_message::ClientMessage, messages_config::MessagesConfig};

pub struct ConnectConfig {
    pub(crate) clean_start: bool,
    pub(crate) last_will_flag: bool,
    pub(crate) last_will_qos: u8,
    pub(crate) last_will_retain: bool,
    pub(crate) keep_alive: u16,
    pub(crate) properties: ConnectProperties,
    pub(crate) client_id: String,
    pub(crate) will_properties: WillProperties,
    pub(crate) last_will_topic: String,
    pub(crate) last_will_message: String,
    pub(crate) username: String,
    pub(crate) password: String,
}

impl MessagesConfig for ConnectConfig {
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
}

#[cfg(test)]
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
