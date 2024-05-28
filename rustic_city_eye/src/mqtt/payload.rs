use std::io::{Error, Read, Write};

use super::client_message::ClientMessage;

pub trait Payload{
    fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()>;
    fn read_from(stream: &mut dyn Read) -> Result<ClientMessage, Error>;
}