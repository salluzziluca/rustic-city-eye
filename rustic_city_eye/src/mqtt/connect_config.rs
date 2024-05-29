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
