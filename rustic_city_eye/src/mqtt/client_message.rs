use std::{io::{BufWriter, Error, Read, Write}, net::TcpStream};

//use self::quality_of_service::QualityOfService;

#[path = "quality_of_service.rs"] mod quality_of_service;


#[derive(Debug)]
pub enum ClientMessage {
    Connect {
        client_id: u32,
        //clean_session: bool,
        //username: String
    },
    Publish {
        packet_id: usize,
        topic_name: String,
        qos: usize,
        retain_flag: bool,
        payload: String,
        dup_flag: bool
    }
}

impl ClientMessage {
    pub fn write_to(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let mut writer = BufWriter::new(stream);
        match self {
            ClientMessage::Connect { client_id } => {
                let client_id_be = client_id.to_be_bytes();
                writer.write(&client_id_be)?;
                writer.flush()?;

                Ok(())
            },
            ClientMessage::Publish { packet_id, topic_name, qos, retain_flag, payload, dup_flag } => {
                println!("Publishing...");

                Ok(())
            },
        }
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<ClientMessage, Error> {
        let mut num_buffer = [0u8; 4];
        stream.read_exact(&mut num_buffer)?;
        
        let client_id = u32::from_be_bytes(num_buffer);

        Ok(ClientMessage::Connect { client_id })
    }
}