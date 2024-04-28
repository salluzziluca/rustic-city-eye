use std::{
    io::{BufWriter, Error, Read, Write},
    net::TcpStream,
};

//use self::quality_of_service::QualityOfService;

#[path = "quality_of_service.rs"]
mod quality_of_service;

#[derive(Debug)]
pub enum ClientMessage {
    Connect {
        //client_id: u32,
        // clean_session: bool,
        // username: String,
        // password: String,
        // lastWillTopic: String,
        // lastWillQoS: u8,
        // lasWillMessage: String,
        // lastWillRetain: bool,
        // keepAlive: u32,
    },
    Publish {
        packet_id: usize,
        topic_name: String,
        qos: usize,
        retain_flag: bool,
        payload: String,
        dup_flag: bool,
    },
}

impl ClientMessage {
    pub fn write_to(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let mut writer = BufWriter::new(stream);
        match self {
            ClientMessage::Connect {
                //client_id,
                // clean_session,
                // username,
                // password,
                // lastWillTopic,
                // lastWillQoS,
                // lasWillMessage,
                // lastWillRetain,
                // keepAlive,
            } => {
                //deberia armar el fixed header
                let byte_1: u8 = 0x10_u8.to_le();

                writer.write(&[byte_1])?;
                writer.flush()?;

                Ok(())
            }
            ClientMessage::Publish {
                packet_id,
                topic_name,
                qos,
                retain_flag,
                payload,
                dup_flag,
            } => {
                println!("Publishing...");

                Ok(())
            }
        }
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<ClientMessage, Error> {
        let mut header = [0u8; 1];
        stream.read_exact(&mut header)?;

        let header = u8::from_be_bytes(header);

        match header {
            0x10 => Ok(ClientMessage::Connect {}),
            _ => Err(Error::new(std::io::ErrorKind::Other, "Invalid header")),
        }
    }
}
