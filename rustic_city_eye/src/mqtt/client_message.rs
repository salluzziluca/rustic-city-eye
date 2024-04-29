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
        client_id: String,
        clean_start: bool,
        last_will_flag: bool, //si el will message tiene que ser guardado asociado a la sesion
        last_will_qos: u8,    //QoS level utilizado cuando se publique el will message
        last_will_retain: bool, // Si el will Message se retiene despues de ser publicado
        username: String,
        password: String,
        keep_alive: u16,
        last_will_delay_interval: u32,
        message_expiry_interval: u16,
        content_type: String,
        user_property: Option<(String, String)>,
        //response_topic: String, //Topic name del response message
        // lastWillTopic: String,
        last_will_message: String,
    },
    Publish {
        //packet_id: usize,
        // topic_name: String,
        // qos: usize,
        // retain_flag: bool,
        // payload: String,
        // dup_flag: bool,
    },
}

impl ClientMessage {
    pub fn write_to(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let mut writer = BufWriter::new(stream);
        match self {
            ClientMessage::Connect {
                client_id,
                clean_start,
                last_will_flag,
                last_will_qos,
                last_will_retain,
                username,
                password,
                keep_alive,
                last_will_delay_interval,
                message_expiry_interval,
                content_type,
                user_property,
                last_will_message,
    
                // lastWillTopic,
                // last_will_qos,
                // lasWillMessage,
                // last_will_retain,
            } => {
                //fixed header
                let byte_1: u8 = 0x10_u8.to_le(); //00010000

                writer.write(&[byte_1])?;

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

                connect_flags |= (last_will_qos & 0b11) << 3;

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

                //keep alive
                let keep_alive = keep_alive.to_le_bytes();
                writer.write(&keep_alive)?;

                //connect properties
                //let mut property_length = 0;
                //TODO: implementar las 300 millones de propiedades, por ahi estría bueno usar un struct

                //payload
                //client ID
                
                let client_id_length = client_id.len() as u16;
                let client_id_length_bytes = client_id_length.to_le_bytes();
                writer.write(&client_id_length_bytes)?;
                writer.write(&client_id.as_bytes())?;

                //will properties
                let will_delay_interval_bytes = last_will_delay_interval.to_le_bytes();
                writer.write(&will_delay_interval_bytes)?;

                // let payload_format_indicator:u8;
                // match std::str::from_utf8(last_will_message) {
                //     Ok(_) => payload_format_indicator = 0x01,
                //     Err(_) => payload_format_indicator = 0x00,
                // }

                let message_expiry_interval_bytes = message_expiry_interval.to_le_bytes();
                writer.write(&message_expiry_interval_bytes)?;
                
                let content_type_length = content_type.len() as u16;
                let content_type_length_bytes = content_type_length.to_le_bytes();
                writer.write(&content_type_length_bytes)?;
                writer.write(&content_type.as_bytes())?;

                //user property
                if let Some((key, value)) = user_property {
                    let key_length = key.len() as u16;
                    let key_length_bytes = key_length.to_le_bytes();
                    writer.write(&key_length_bytes)?;
                    writer.write(&key.as_bytes())?;

                    let value_length = value.len() as u16;
                    let value_length_bytes = value_length.to_le_bytes();
                    writer.write(&value_length_bytes)?;
                    writer.write(&value.as_bytes())?;
                }

                //will payload

                let will_message_length = last_will_message.len() as u16;
                let will_message_length_bytes = will_message_length.to_le_bytes();
                writer.write(&will_message_length_bytes)?;

                writer.write(&last_will_message.as_bytes())?;

                writer.flush()?;
                Ok(())
            }
            ClientMessage::Publish {
                // packet_id,
                // topic_name,
                // qos,
                // retain_flag,
                // payload,
                // dup_flag,
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

                //keep alive
                let mut keep_alive_buf = [0u8; 2];
                stream.read_exact(&mut keep_alive_buf)?;
                let keep_alive = u16::from_le_bytes(keep_alive_buf);

                // connect properties


                //payload
                //client ID
                let mut client_id_length_buf = [0u8; 2];
                stream.read_exact(&mut client_id_length_buf)?;
                let client_id_length = u16::from_le_bytes(client_id_length_buf);
                let mut client_id_buf = vec![0; client_id_length as usize];
                stream.read_exact(&mut client_id_buf)?;
                let client_id = std::str::from_utf8(&client_id_buf).expect("Error al leer client_id");

                //will properties
                let mut will_delay_interval_buf = [0u8; 4];
                stream.read_exact(&mut will_delay_interval_buf)?;
                let will_delay_interval = u32::from_le_bytes(will_delay_interval_buf);

                let mut message_expiry_interval_buf = [0u8; 2];
                stream.read_exact(&mut message_expiry_interval_buf)?;
                let message_expiry_interval = u16::from_le_bytes(message_expiry_interval_buf);

                let mut content_type_length_buf = [0u8; 2];
                stream.read_exact(&mut content_type_length_buf)?;
                let content_type_length = u16::from_le_bytes(content_type_length_buf);
                let mut content_type_buf = vec![0; content_type_length as usize];
                stream.read_exact(&mut content_type_buf)?;
                let content_type = std::str::from_utf8(&content_type_buf).expect("Error al leer content_type");

                //user property
                let mut user_property_key_length_buf = [0u8; 2];
                stream.read_exact(&mut user_property_key_length_buf)?;
                let user_property_key_length = u16::from_le_bytes(user_property_key_length_buf);
                let mut user_property_key_buf = vec![0; user_property_key_length as usize];
                stream.read_exact(&mut user_property_key_buf)?;
                let user_property_key = std::str::from_utf8(&user_property_key_buf).expect("Error al leer user_property_key");

                let mut user_property_value_length_buf = [0u8; 2];
                stream.read_exact(&mut user_property_value_length_buf)?;
                let user_property_value_length = u16::from_le_bytes(user_property_value_length_buf);
                let mut user_property_value_buf = vec![0; user_property_value_length as usize];
                stream.read_exact(&mut user_property_value_buf)?;
                let user_property_value = std::str::from_utf8(&user_property_value_buf).expect("Error al leer user_property_value");

                //will payload
                let mut will_message_length_buf = [0u8; 2];
                stream.read_exact(&mut will_message_length_buf)?;
                let will_message_length = u16::from_le_bytes(will_message_length_buf);
                let mut will_message_buf = vec![0; will_message_length as usize];
                stream.read_exact(&mut will_message_buf)?;
                let will_message = std::str::from_utf8(&will_message_buf).expect("Error al leer will_message");


                Ok(ClientMessage::Connect {
                    clean_start: (connect_flags & (1 << 1)) != 0,
                    last_will_flag: (connect_flags & (1 << 2)) != 0,
                    last_will_qos: (connect_flags >> 3) & 0b11,
                    last_will_retain: (connect_flags & (1 << 5)) != 0,
                    username: user.to_string(),
                    password: pass.to_string(),
                    keep_alive: keep_alive,
                    client_id: client_id.to_string(),
                    last_will_delay_interval: will_delay_interval,
                    message_expiry_interval: message_expiry_interval,
                    content_type: content_type.to_string(),
                    user_property: Some((user_property_key.to_string(), user_property_value.to_string())),
                    last_will_message: will_message.to_string(),
            
                    
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
