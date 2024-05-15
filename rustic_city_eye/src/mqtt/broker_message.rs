use std::io::{BufWriter, Error, Read, Write};

use super::{reader::{read_string, read_u8}, writer::{write_string, write_u8}};

#[derive(Debug, PartialEq)]
pub enum BrokerMessage {
    Connack {
        //session_present: bool,
        //return_code: u32
    },
    Puback {
        packet_id_msb: u8,
        packet_id_lsb: u8,
        reason_code: u8,
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
    PublishDelivery {
        payload: String
    }
}
#[allow(dead_code)]
impl BrokerMessage {
    pub fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        let mut writer = BufWriter::new(stream);
        match self {
            BrokerMessage::Connack {} => {
                let byte_1: u8 = 0x10_u8.to_le();

                writer.write_all(&[byte_1])?;
                writer.flush()?;

                Ok(())
            }
            BrokerMessage::Puback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
            } => {
                //fixed header
                let byte_1: u8 = 0x40_u8.to_le(); //01000000

                writer.write_all(&[byte_1])?;

                //variable header
                //packet_id
                write_u8(&mut writer, packet_id_msb)?;
                write_u8(&mut writer, packet_id_lsb)?;

                writer.flush()?;

                Ok(())
            }
            BrokerMessage::Suback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
            } => {
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
            }
            BrokerMessage::PublishDelivery { payload } => {
                //fixed header -> es uno de juguete, hay que pensarlo mejor
                let byte_1: u8 = 0x00_u8.to_le(); 
                writer.write_all(&[byte_1])?;

                write_string(&mut writer, &payload)?;
                writer.flush()?;

                Ok(())
            },
        }
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<BrokerMessage, Error> {
        let mut header = [0u8; 1];
        stream.read_exact(&mut header)?;
        let header = u8::from_le_bytes(header); //BUG:

        match header {
            0x00 => {
                let payload = read_string(stream)?;

                Ok(BrokerMessage::PublishDelivery { payload })
            }
            0x10 => Ok(BrokerMessage::Connack {}),
            0x40 => {
                let packet_id_msb = read_u8(stream)?;
                let packet_id_lsb = read_u8(stream)?;
                Ok(BrokerMessage::Puback {
                    packet_id_msb,
                    packet_id_lsb,
                    reason_code: 1,
                })
            }
            0x90 => {
                let packet_id_msb = read_u8(stream)?;
                let packet_id_lsb = read_u8(stream)?;
                Ok(BrokerMessage::Suback {
                    packet_id_msb,
                    packet_id_lsb,
                    reason_code: 1,
                })
            }
            _ => Err(Error::new(std::io::ErrorKind::Other, "Invalid header")),
        }
    }

    pub fn analize_packet_id(&self, packet_id: u16) -> bool {
        match self {
            BrokerMessage::Connack {  } => true,
            BrokerMessage::Puback { packet_id_msb, packet_id_lsb, reason_code: _ } => {
                let bytes = packet_id.to_be_bytes();

                if bytes[0] == *packet_id_msb && bytes[1] == *packet_id_lsb {
                    return true;
                }
                false
            },
            BrokerMessage::Suback { packet_id_msb, packet_id_lsb, reason_code: _ } => {
                let bytes = packet_id.to_be_bytes();

                if bytes[0] == *packet_id_msb && bytes[1] == *packet_id_lsb {
                    return true;
                }
                false
            }
            BrokerMessage::PublishDelivery { payload } => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_01_suback_ok() {
        let suback = BrokerMessage::Suback {
            reason_code: 1,
            packet_id_msb: 1,
            packet_id_lsb: 1,
        };

        let mut cursor = Cursor::new(Vec::<u8>::new());
        suback.write_to(&mut cursor).unwrap();
        cursor.set_position(0);
        let read_suback = BrokerMessage::read_from(&mut cursor).unwrap();
        assert_eq!(suback, read_suback);
    }

    #[test]
    fn test_02_analizing_packet_ids_ok () {
        let suback = BrokerMessage::Suback {
            reason_code: 1,
            packet_id_msb: 2,
            packet_id_lsb: 1,
        };

        let puback = BrokerMessage::Puback { packet_id_msb: 1, packet_id_lsb: 5, reason_code: 1 };

        assert!(suback.analize_packet_id(513));
        assert!(puback.analize_packet_id(261));

    }
}
