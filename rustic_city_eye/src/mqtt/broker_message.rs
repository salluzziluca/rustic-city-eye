use std::{io::{BufWriter, Error, Read, Write}, net::TcpStream};

#[derive(Debug)]
pub enum BrokerMessage {
    Connack {
        session_present: bool,
        return_code: u32
    },
}

impl BrokerMessage {
    pub fn write_to(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let mut writer = BufWriter::new(stream);
        match self {
            BrokerMessage::Connack { session_present, return_code } => {
                let mut session_present_usize: u32  = 0; //false

                if *session_present {
                    session_present_usize = 1;
                }

                let session_present_be = session_present_usize.to_be_bytes();
                writer.write(&session_present_be)?;
                writer.flush()?;

                let return_code_be = return_code.to_be_bytes();
                writer.write(&return_code_be)?;
                writer.flush()?;

                Ok(())
            },
        }
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<BrokerMessage, Error> {
        let mut num_buffer = [0u8; 4];
        stream.read_exact(&mut num_buffer)?;
        
        let session_present_usize = u32::from_be_bytes(num_buffer);
        let mut session_present = false;

        if session_present_usize == 1 {
            session_present = true;
        }

        let mut num_buffer = [0u8; 4];
        stream.read_exact(&mut num_buffer)?;
        
        let return_code = u32::from_be_bytes(num_buffer);

        Ok(BrokerMessage::Connack { session_present, return_code })
    }
}