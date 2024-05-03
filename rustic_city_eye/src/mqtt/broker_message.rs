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
    Suback {
        reason_code: u8,
    },
}
#[allow(dead_code)]
impl BrokerMessage {
    pub fn write_to(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let mut writer = BufWriter::new(stream);
        match self {
            BrokerMessage::Connack {} => {
                let byte_1: u8 = 0x10_u8.to_le();

                writer.write(&[byte_1])?;
                writer.flush()?;

                Ok(())
            },
            BrokerMessage::Suback { reason_code: _ } => {
                println!("Entra a suback");
                //fixed header
                let byte_1: u8 = 0x90_u8.to_le(); //10010000

                writer.write(&[byte_1])?;

                //variable header
                let byte_2: u8 = 0x00_u8.to_le(); //00000000
                writer.write(&[byte_2])?;

                //payload
                let byte_3: u8 = 0x01_u8.to_le(); //00000001
                writer.write(&[byte_3])?;

                writer.flush()?;

                Ok(())
            },
        }
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<BrokerMessage, Error> {
        println!("Entra a read_from"); // hasta acá llega
        let mut header = [0u8; 1];
        println!("Header = {:?}", header);
        stream.read_exact(&mut header)?;

        let header = u8::from_le_bytes(header);
        println!("Header: {:?}", header);


        match header {
            0x10 => Ok(BrokerMessage::Connack {}),
            0x90 =>{ 
                println!("Entra a suback"); // acá no llega
                Ok(BrokerMessage::Suback { reason_code: 0 })},
            _ => Err(Error::new(std::io::ErrorKind::Other, "Invalid header")),
        }
    
    }
}
