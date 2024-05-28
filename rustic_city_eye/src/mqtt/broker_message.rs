use std::io::{BufWriter, Error, Read, Write};

use crate::helpers::payload_types::PayloadTypes;

use super::payload::Payload;
use super::{reader::*, writer::*};

const SESSION_EXPIRY_INTERVAL_ID: u8 = 0x11;
const REASON_STRING_ID: u8 = 0x1F;
const USER_PROPERTY_ID: u8 = 0x26;
use super::{
    publish_properties::PublishProperties,
    reader::{read_string, read_u8},
    writer::{write_string, write_u8},
};

#[derive(Debug, PartialEq)]
pub enum BrokerMessage {
    Connack {
        //session_present: bool,
        //return_code: u32
    },

    /// Puback es la respuesta a un Publish packet con QoS 1.
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
        sub_id: u8,
    },
    PublishDelivery {
        packet_id: u16,
        topic_name: String,
        qos: usize,
        retain_flag: usize,
        payload: PayloadTypes,
        dup_flag: usize,
        properties: PublishProperties,
    },
    Unsuback {
        packet_id_msb: u8,
        packet_id_lsb: u8,
        reason_code: u8,
    },
    Disconnect {
        reason_code: u8,
        session_expiry_interval: u32,
        reason_string: String,
        user_properties: Vec<(String, String)>,
    },
    Pingresp,
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
                reason_code,
            } => {
                //fixed header
                let byte_1: u8 = 0x40_u8.to_le(); //01000000

                writer.write_all(&[byte_1])?;

                //variable header
                //packet_id
                write_u8(&mut writer, packet_id_msb)?;
                write_u8(&mut writer, packet_id_lsb)?;

                //reason code
                write_u8(&mut writer, reason_code)?;

                writer.flush()?;

                Ok(())
            }
            BrokerMessage::Suback {
                packet_id_msb,
                packet_id_lsb,
                reason_code,
                sub_id,
            } => {
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

                //reason code
                write_u8(&mut writer, reason_code)?;

                //sub_id
                write_u8(&mut writer, sub_id)?;
                writer.flush()?;

                Ok(())
            }
            BrokerMessage::PublishDelivery {
                packet_id,
                topic_name,
                qos,
                retain_flag,
                payload,
                dup_flag,
                properties,
            } => {
                //fixed header -> es uno de juguete, hay que pensarlo mejor
                let byte_1: u8 = 0x00_u8.to_le();
                writer.write_all(&[byte_1])?;

                //variable header
                //packet_id
                write_u8(&mut writer, &packet_id.to_be_bytes()[0])?;
                write_u8(&mut writer, &packet_id.to_be_bytes()[1])?;

                //topic_name
                write_string(&mut writer, topic_name)?;

                //qos
                write_u8(&mut writer, &qos.to_be_bytes()[0])?;

                //retain_flag
                write_u8(&mut writer, &retain_flag.to_be_bytes()[0])?;

                //payload
                payload.write_to(&mut writer)?;
                // write_string(&mut writer, payload)?;

                //dup_flag
                write_u8(&mut writer, &dup_flag.to_be_bytes()[0])?;

                //properties
                properties.write_properties(&mut writer)?;

                writer.flush()?;

                Ok(())
            }
            BrokerMessage::Unsuback {
                packet_id_msb,
                packet_id_lsb,
                reason_code,
            } => {
                //fixed header
                let byte_1: u8 = 0xB0_u8.to_le(); //10110000

                writer.write_all(&[byte_1])?;

                //variable header
                //packet_id
                write_u8(&mut writer, packet_id_msb)?;
                write_u8(&mut writer, packet_id_lsb)?;
                write_u8(&mut writer, reason_code)?;

                writer.flush()?;

                Ok(())
            }
            BrokerMessage::Disconnect {
                reason_code,
                session_expiry_interval,
                reason_string,
                user_properties,
            } => {
                //fixed header
                let header: u8 = 0xE0_u8.to_le(); //11100000
                write_u8(&mut writer, &header)?;
                //variable_header
                write_u8(&mut writer, reason_code)?;

                write_u8(&mut writer, &SESSION_EXPIRY_INTERVAL_ID)?;
                write_u32(&mut writer, session_expiry_interval)?;

                write_u8(&mut writer, &REASON_STRING_ID)?;
                write_string(&mut writer, reason_string)?;

                write_u8(&mut writer, &USER_PROPERTY_ID)?;
                write_string_pairs(&mut writer, user_properties)?;
                writer.flush()?;
                Ok(())
            }
            BrokerMessage::Pingresp => {
                let byte_1: u8 = 0xD0_u8.to_le();
                writer.write_all(&[byte_1])?;
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
            0x00 => {
                let packet_id_msb = read_u8(stream)?;
                let packet_id_lsb = read_u8(stream)?;
                let topic_name = read_string(stream)?;
                let qos = read_u8(stream)? as usize;
                let retain_flag = read_u8(stream)? as usize;
                let payload = PayloadTypes::read_from(stream)?;

                let dup_flag = read_u8(stream)? as usize;
                let properties = PublishProperties::read_from(stream)?;

                Ok(BrokerMessage::PublishDelivery {
                    packet_id: u16::from_be_bytes([packet_id_msb, packet_id_lsb]),
                    topic_name,
                    qos,
                    retain_flag,
                    payload,
                    dup_flag,
                    properties,
                })
            }
            0x10 => Ok(BrokerMessage::Connack {}),
            0x40 => {
                let packet_id_msb = read_u8(stream)?;
                let packet_id_lsb = read_u8(stream)?;
                let reason_code = read_u8(stream)?;

                Ok(BrokerMessage::Puback {
                    packet_id_msb,
                    packet_id_lsb,
                    reason_code,
                })
            }
            0x90 => {
                let packet_id_msb = read_u8(stream)?;
                let packet_id_lsb = read_u8(stream)?;
                let reason_code = read_u8(stream)?;
                let sub_id = read_u8(stream)?;
                Ok(BrokerMessage::Suback {
                    packet_id_msb,
                    packet_id_lsb,
                    reason_code,
                    sub_id,
                })
            }
            0xB0 => {
                let packet_id_msb = read_u8(stream)?;
                let packet_id_lsb = read_u8(stream)?;
                let reason_code = read_u8(stream)?;

                Ok(BrokerMessage::Unsuback {
                    packet_id_msb,
                    packet_id_lsb,
                    reason_code,
                })
            }
            0xE0 => {
                let reason_code = read_u8(stream)?;
                let session_expiry_interval_id = read_u8(stream)?;
                if session_expiry_interval_id != SESSION_EXPIRY_INTERVAL_ID {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid session expiry interval id",
                    ));
                }
                let session_expiry_interval = read_u32(stream)?;

                let reason_string_id = read_u8(stream)?;
                if reason_string_id != REASON_STRING_ID {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid reason string id",
                    ));
                }
                let reason_string = read_string(stream)?;

                let user_property_id = read_u8(stream)?;
                if user_property_id != USER_PROPERTY_ID {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid user property id",
                    ));
                }
                let user_properties = read_string_pairs(stream)?;

                Ok(BrokerMessage::Disconnect {
                    reason_code,
                    session_expiry_interval,
                    reason_string,
                    user_properties,
                })
            }
            0xD0 => Ok(BrokerMessage::Pingresp),
            _ => Err(Error::new(std::io::ErrorKind::Other, "Invalid header")),
        }
    }

    pub fn analize_packet_id(&self, packet_id: u16) -> bool {
        match self {
            BrokerMessage::Connack {} => true,
            BrokerMessage::Puback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
            } => {
                let bytes = packet_id.to_be_bytes();

                bytes[0] == *packet_id_msb && bytes[1] == *packet_id_lsb
            }
            BrokerMessage::Suback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
                sub_id: _,
            } => {
                let bytes = packet_id.to_be_bytes();

                bytes[0] == *packet_id_msb && bytes[1] == *packet_id_lsb
            }
            BrokerMessage::PublishDelivery {
                packet_id: _,
                topic_name: _,
                qos: _,
                retain_flag: _,
                dup_flag: _,
                properties: _,
                payload: _,
            } => true,
            BrokerMessage::Unsuback {
                packet_id_msb,
                packet_id_lsb,
                reason_code: _,
            } => {
                let bytes = packet_id.to_be_bytes();

                bytes[0] == *packet_id_msb && bytes[1] == *packet_id_lsb
            }
            BrokerMessage::Pingresp => true,
            BrokerMessage::Disconnect {
                reason_code: _,
                session_expiry_interval: _,
                reason_string: _,
                user_properties: _,
            } => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_02_analizing_packet_ids_ok() {
        let suback = BrokerMessage::Suback {
            reason_code: 1,
            packet_id_msb: 2,
            packet_id_lsb: 1,
            sub_id: 1,
        };

        let puback = BrokerMessage::Puback {
            packet_id_msb: 1,
            packet_id_lsb: 5,
            reason_code: 1,
        };

        assert!(suback.analize_packet_id(513));
        assert!(puback.analize_packet_id(261));
    }

    #[test]
    fn test_03_unsuback_ok() {
        let unsuback = BrokerMessage::Unsuback {
            packet_id_msb: 1,
            packet_id_lsb: 1,
            reason_code: 1,
        };

        let mut cursor = Cursor::new(Vec::<u8>::new());
        match unsuback.write_to(&mut cursor) {
            Ok(_) => {}
            Err(err) => {
                println!("Error: {:?}", err);
                panic!();
            }
        }
        cursor.set_position(0);
        let read_unsuback = match BrokerMessage::read_from(&mut cursor) {
            Ok(unsuback) => unsuback,
            Err(err) => {
                println!("Error: {:?}", err);

                panic!()
            }
        };
        assert_eq!(unsuback, read_unsuback);
    }
}
