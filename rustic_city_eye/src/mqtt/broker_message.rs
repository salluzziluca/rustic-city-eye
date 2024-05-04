use std::{
    io::{BufWriter, Error, Read, Write},
    net::TcpStream,
};

use super::{reader::read_u8, writer::write_u8};

#[derive(Debug, PartialEq)]
pub enum BrokerMessage {
    Connack {
        //session_present: bool,
        //return_code: u32
    },
    /// El Suback se utiliza para confirmar la suscripción a un topic
    /// 
    /// reason_code es el código de razón de la confirmación
    /// packet_id_msb y packet_id_lsb son los bytes más significativos y menos significativos del packet_id
    Suback {
        /// packet_id_msb es el byte más significativo del packet_id
        packet_id_msb: u8,
        /// packet_id_lsb es el byte menos significativo del packet_id
        packet_id_lsb: u8,
        /// reason_code es el código de razón de la confirmación
        reason_code: u8,
    },
}
#[allow(dead_code)]
impl BrokerMessage {
    pub fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        let mut writer = BufWriter::new(stream);
        match self {
            BrokerMessage::Connack {} => {
                let byte_1: u8 = 0x10_u8.to_le();

                writer.write(&[byte_1])?;
                writer.flush()?;

                Ok(())
            },
            BrokerMessage::Suback { packet_id_msb, packet_id_lsb, reason_code: _ } => {
                println!("Subacking...");
                //fixed header
                let byte_1: u8 = 0x90_u8.to_le(); //10010000

                writer.write_all(&[byte_1])?;

                //variable header
                //let byte_2: u8 = 0x00_u8.to_le(); //00000000
                //writer.write(&[byte_2])?;
                
               
                //payload
                //let byte_3: u8 = 0x01_u8.to_le(); //00000001
                //writer.write(&[byte_3])?;
                write_u8(&mut writer, packet_id_msb)?;
                write_u8(&mut writer, packet_id_lsb)?;
                writer.flush()?;

                Ok(())
            },
        }
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<BrokerMessage, Error> {
        let mut header = [0u8; 1];
        stream.read_exact(&mut header)?;

        let header = u8::from_le_bytes(header);

        match header {
            0x10 => Ok(BrokerMessage::Connack {}),
            0x90 => {
                let packet_id_msb = read_u8(stream)?;
                let packet_id_lsb = read_u8(stream)?;
                Ok(BrokerMessage::Suback {
                    packet_id_msb,
                    packet_id_lsb,
                    reason_code: 1,
                })
            },
            _ => Err(Error::new(std::io::ErrorKind::Other, "Invalid header")),
        }
        
        
    }
}
