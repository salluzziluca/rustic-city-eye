use std::{
    io::{BufWriter, Error, Read, Write},
    net::TcpStream,
};

//use self::quality_of_service::QualityOfService;
const PROTOCOL_VERSION: u8 = 5;
#[path = "quality_of_service.rs"]
mod quality_of_service;

#[derive(Debug)]
//implement partial eq
#[derive(PartialEq)]
pub enum ClientMessage {
    Connect {
        //client_id: u32,
        clean_start: bool,
        last_will_flag: bool, //si el will message tiene que ser guardado asociado a la sesion
        last_will_QoS: u8,    //QoS level utilizado cuando se publique el will message
        last_will_retain: bool, // Si el will Message se retiene despues de ser publicado
        username: String,
        password: String,
        // lastWillTopic: String,
        // lasWillMessage: String,
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
                clean_start,
                last_will_flag,
                last_will_QoS,
                last_will_retain,
                username,
                password,
                // username,
                // password,
                // lastWillTopic,
                // last_will_QoS,
                // lasWillMessage,
                // last_will_retain,
                // keepAlive,
            } => {
                //fixed header
                let byte_1: u8 = 0x10_u8.to_le(); //00010000

                writer.write(&[byte_1])?;
                writer.flush()?;

                //protocol name
                let protocol_name = "MQTT";
                let protocol_name_length = protocol_name.len() as u16;
                let protocol_name_length_bytes = protocol_name_length.to_le_bytes();
                writer.write(&[protocol_name_length_bytes[0]])?;
                writer.write(&[protocol_name_length_bytes[1]])?;
                writer.write(&protocol_name.as_bytes())?;

                //protocol version
                let protocol_version: u8 = 0x05;
                writer.write(&[protocol_version])?;

                //connection flags
                let mut connect_flags: u8 = 0x00;
                if *clean_start {
                    connect_flags |= 1 << 1; //set bit 1 to 1
                }

                if *last_will_flag {
                    connect_flags |= 1 << 2;
                }

                connect_flags |= (last_will_QoS & 0b11) << 3;

                if *last_will_retain {
                    connect_flags |= 1 << 5;
                }

                if password.len() != 0 {
                    connect_flags |= 1 << 6;
                }

                if username.len() != 0 {
                    connect_flags |= 1 << 7;
                }

                writer.write(&[connect_flags])?;
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

        let header = u8::from_le_bytes(header);

        match header {
            0x10 => {
                //leo el protocol name
                let mut protocol_lenght_buf = [0u8; 2];
                stream.read_exact(&mut protocol_lenght_buf)?;
                let protocol_lenght = u16::from_le_bytes(protocol_lenght_buf);
                println!("protocol_lenght: {:?}", protocol_lenght);

                let mut protocol_name_buf = vec![0; protocol_lenght as usize];
                stream.read_exact(&mut protocol_name_buf)?;

                let protocol_name =
                    std::str::from_utf8(&protocol_name_buf).expect("Error al leer protocol_name");
                println!("protocol_name: {:?}", protocol_name);

                if protocol_name != "MQTT" {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid protocol name",
                    ));
                }

                //protocol version
                let mut protocol_version_buf = [0u8; 1];
                stream.read_exact(&mut protocol_version_buf)?;
                let protocol_version = u8::from_le_bytes(protocol_version_buf);

                if protocol_version != PROTOCOL_VERSION {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid protocol version",
                    ));
                }
                println!("protocol version: {:?}", protocol_version);

                //connect flags
                let mut connect_flags_buf = [0u8; 1];
                stream.read_exact(&mut connect_flags_buf)?;
                let connect_flags = u8::from_le_bytes(connect_flags_buf);

                let hay_user = (connect_flags & (1 << 7)) != 0;
                let mut user = "";
                if hay_user {
                    user = "user";
                }
                let hay_pass = (connect_flags & (1 << 6)) != 0;
                let mut pass = "";
                if hay_pass {
                    pass = "pass";
                }

                Ok(ClientMessage::Connect {
                    clean_start: (connect_flags & (1 << 1)) != 0,
                    last_will_flag: (connect_flags & (1 << 2)) != 0,
                    last_will_QoS: (connect_flags >> 3) & 0b11,
                    last_will_retain: (connect_flags & (1 << 5)) != 0,
                    username: user.to_string(),
                    password: pass.to_string(),
                })
            }
            _ => Err(Error::new(std::io::ErrorKind::Other, "Invalid header")),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn read_from_connect() {
//         let mut stream = std::io::Cursor::new(vec![0x10, 0x00, 0x04, b'M', b'Q', b'T', b'T']);
//         let message = ClientMessage::read_from(&mut stream).unwrap();
//         let expected = ClientMessage::Connect {};
//         assert_eq!(message, expected);
//     }
// }
