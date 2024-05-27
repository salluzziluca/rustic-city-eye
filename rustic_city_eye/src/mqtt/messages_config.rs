use crate::mqtt::client_message::ClientMessage;

pub trait MessagesConfig {
    fn parse_message(&self, packet_id: u16) -> ClientMessage;
}
