use crate::{client_message::ClientMessage, messages_config::MessagesConfig};

pub struct DisconnectConfig {
    pub reason_code: u8,
    pub session_expiry_interval: u32,
    pub reason_string: String,
    pub client_id: String,
}

impl MessagesConfig for DisconnectConfig {
    fn parse_message(&self, _packet_id: u16) -> ClientMessage {
        ClientMessage::Disconnect {
            reason_code: self.reason_code,
            session_expiry_interval: self.session_expiry_interval,
            reason_string: self.reason_string.clone(),
            client_id: self.client_id.clone(),
        }
    }
}

impl DisconnectConfig {
    pub fn new(
        reason_code: u8,
        session_expiry_interval: u32,
        reason_string: String,
        client_id: String,
    ) -> DisconnectConfig {
        DisconnectConfig {
            reason_code,
            session_expiry_interval,
            reason_string,
            client_id,
        }
    }
}