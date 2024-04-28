use std::{
    io::{BufWriter, Error, Read, Write},
    net::TcpStream,
};

#[derive(Debug)]
pub enum BrokerMessage {
    Connack {
        //session_present: bool,
        //return_code: u32
    },
}

impl BrokerMessage {
    pub fn write_to(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let mut writer = BufWriter::new(stream);
        match self {
            BrokerMessage::Connack {} => {
                let byte_1: u8 = 0x10_u8.to_le();

                writer.write(&[byte_1])?;
                writer.flush()?;

                Ok(())
            }
        }
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<BrokerMessage, Error> {
        let mut header = [0u8; 1];
        stream.read_exact(&mut header)?;

        let header = u8::from_le_bytes(header);

        match header {
            0x10 => Ok(BrokerMessage::Connack {}),
            _ => Err(Error::new(std::io::ErrorKind::Other, "Invalid header")),
        }
    }
}