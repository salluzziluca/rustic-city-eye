use super::{broker_message::BrokerMessage, client_message::ClientMessage};

pub enum StreamOperation {
    WriteClientMessage(ClientMessage),
    WriteAndDisconnect(ClientMessage),
    WriteBrokerMessage(BrokerMessage),
    ShutdownStream,
}
