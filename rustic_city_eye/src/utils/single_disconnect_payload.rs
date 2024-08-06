use serde::{Deserialize, Serialize};

use crate::mqtt::protocol_error::ProtocolError;

use super::writer::{write_string, write_u32, write_u8};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SingleDisconnectPayload {
    id: u32,
}

impl SingleDisconnectPayload {
    pub fn new(id: u32) -> SingleDisconnectPayload {
        SingleDisconnectPayload { id }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn write_to(&self, stream: &mut dyn std::io::prelude::Write) -> Result<(), ProtocolError> {
        write_u32(stream, &self.id)?;

        Ok(())
    }
}
