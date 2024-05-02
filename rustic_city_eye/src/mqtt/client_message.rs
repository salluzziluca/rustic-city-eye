use crate::mqtt::reader::*;
use crate::mqtt::will_properties::*;
use crate::mqtt::writer::*;

use std::io::{BufWriter, Error, Read, Write};

use super::connect_propierties::ConnectProperties;

//use self::quality_of_service::QualityOfService;
const PROTOCOL_VERSION: u8 = 5;

#[path = "quality_of_service.rs"]
mod quality_of_service;

#[derive(Debug)]
//implement partial eq
#[derive(PartialEq)]

pub enum ClientMessage {
    ///El Connect Message es el primer menasje que el cliente envia cuando se conecta al broker. Este contiene toda la informacion necesaria para que el broker identifique al cliente y pueda establecer una sesion con los parametros establecidos.
    ///
    /// clean_start especifica si se debe limpiar la sesion previa del cliente y arrancar una nueva limpia y desde cero.
    ///
    /// last_will_flag especifica si el will message se debe guardar asociado a la sesion, last_will_qos especifica el QoS level utilizado cuando se publique el will message, last_will_retain especifica si el will message se retiene despues de ser publicado.
    ///
    /// keep_alive especifica el tiempo en segundos que el broker debe esperar entre mensajes del cliente antes de desconectarlo.
    ///
    /// Si el will flag es true, se escriben el will topic y el will message.
    ///
    /// finalmente, si el cliente envia un username y un password, estos se escriben en el payload.
    Connect {
        clean_start: bool,
        last_will_flag: bool,
        last_will_qos: u8,
        last_will_retain: bool,
        keep_alive: u16,
        // properties: ConnectProperties,
        client_id: String,
        will_properties: WillProperties,
        last_will_topic: String,
        last_will_message: String,
        username: String,
        password: String,
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

#[allow(dead_code)]
impl ClientMessage {
    pub fn write_to(&self, stream: &mut dyn Write) -> Result<(), Error> {
        let mut writer = BufWriter::new(stream);
        match self {
            ClientMessage::Connect {
                client_id,
                clean_start,
                last_will_flag,
                last_will_qos,
                last_will_retain,
                keep_alive,
                // properties,
                will_properties,
                last_will_topic,
                last_will_message,
                username,
                password,
            } => {
                //fixed header
                let byte_1: u8 = 0x10_u8.to_le(); //00010000
                writer.write(&[byte_1])?;

                //TODO: aca deberia ir el remaining lenght field
                //protocol name
                let protocol_name = "MQTT";
                write_string(&mut writer, protocol_name)?;
                

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
                if *last_will_qos > 3 {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid last will qos",
                    ));
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
                write_u16(&mut writer, keep_alive)?; 

                //connect properties
                // properties.write_to(&mut writer)?;
                //let mut property_length = 0;
                //TODO: implementar las 300 millones de propiedades, por ahi estría bueno usar un struct

                //payload
                write_string(&mut writer, &client_id)?;
                will_properties.write_to(&mut writer)?;

                if *last_will_flag {
                    write_string(&mut writer, &last_will_topic)?;
                    write_string(&mut writer, &last_will_message)?;

                }

                if username.len() != 0 {
                    write_string(&mut writer, &username)?;
                }
                if password.len() != 0 {
                    write_string(&mut writer, &password)?;
                }

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
                let protocol_name = read_string(stream)?;

                if protocol_name != "MQTT" {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid protocol name",
                    ));
                }

                //protocol version
                let protocol_version = read_u8(stream)?;

                if protocol_version != PROTOCOL_VERSION {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid protocol version",
                    ));
                }

                //connect flags
                let connect_flags = read_u8(stream)?;
                let clean_start = (connect_flags & (1 << 1)) != 0;
                let last_will_flag = (connect_flags & (1 << 2)) != 0;
                let last_will_qos = (connect_flags >> 3) & 0b11;
                let last_will_retain = (connect_flags & (1 << 5)) != 0;

                //keep alive
                let keep_alive = read_u16(stream)?;

                //properties
                let properties = ConnectProperties::read_from(stream)?;

                //payload
                //client ID
                let client_id = read_string(stream)?;

                let will_properties = WillProperties::read_from(stream)?;
                let mut last_will_topic = String::new();
                let mut will_message = String::new();
                if last_will_flag {
                    last_will_topic = read_string(stream)?;
                    will_message = read_string(stream)?;
                }

                let hay_user = (connect_flags & (1 << 7)) != 0;
                let mut user = String::new();
                if hay_user {
                    user = read_string(stream)?;
                }

                let hay_pass = (connect_flags & (1 << 6)) != 0;
                let mut pass = String::new();
                if hay_pass {
                    pass = read_string(stream)?;
                }

                Ok(ClientMessage::Connect {
                    client_id: client_id.to_string(),
                    clean_start: clean_start,
                    last_will_flag: last_will_flag,
                    last_will_qos: last_will_qos,
                    last_will_retain: last_will_retain,
                    keep_alive: keep_alive,
                    // properties: properties,
                    will_properties: will_properties,
                    last_will_topic: last_will_topic.to_string(),
                    last_will_message: will_message.to_string(),
                    username: user.to_string(),
                    password: pass.to_string(),
                })
            }
            _ => Err(Error::new(std::io::ErrorKind::Other, "Invalid header")),
        }
    }
}
