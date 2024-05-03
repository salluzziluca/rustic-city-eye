use std::io::{BufWriter, Error, Read, Write};

use crate::mqtt::connect_properties::ConnectProperties;
use crate::mqtt::publish_properties::PublishProperties;
use crate::mqtt::reader::*;
use crate::mqtt::will_properties::*;
use crate::mqtt::writer::*;

use super::publish_properties::TopicProperties;

//use self::quality_of_service::QualityOfService;
const PROTOCOL_VERSION: u8 = 5;

#[derive(Debug, PartialEq)]
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
        properties: ConnectProperties,
        client_id: String,
        will_properties: WillProperties,
        last_will_topic: String,
        last_will_message: String,
        username: String,
        password: String,
    },
    Publish {
        packet_id: u16,
        topic_name: String,
        qos: usize,
        retain_flag: bool,
        payload: String,
        dup_flag: bool,
        properties: PublishProperties,
    },
}

#[allow(dead_code)]
impl ClientMessage {
    pub fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        let mut writer = BufWriter::new(stream);
        match self {
            ClientMessage::Connect {
                client_id,
                clean_start,
                last_will_flag,
                last_will_qos,
                last_will_retain,
                keep_alive,
                properties,
                will_properties,
                last_will_topic,
                last_will_message,
                username,
                password,
            } => {
                //fixed header
                let byte_1: u8 = 0x10_u8.to_le(); //00010000
                writer.write_all(&[byte_1])?;

                //TODO: aca deberia ir el remaining lenght field
                //protocol name
                let protocol_name = "MQTT";
                write_string(&mut writer, protocol_name)?;

                //protocol version
                let protocol_version: u8 = 0x05;
                writer.write_all(&[protocol_version])?;

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

                if !password.is_empty() {
                    connect_flags |= 1 << 6;
                }

                if !username.is_empty() {
                    connect_flags |= 1 << 7;
                }

                writer.write_all(&[connect_flags])?;

                //keep alive
                write_u16(&mut writer, keep_alive)?;
                write_string(&mut writer, client_id)?;

                will_properties.write_to(&mut writer)?;

                if *last_will_flag {
                    write_string(&mut writer, last_will_topic)?;
                    write_string(&mut writer, last_will_message)?;
                }

                if !username.is_empty() {
                    write_string(&mut writer, username)?;
                }
                if !password.is_empty() {
                    write_string(&mut writer, password)?;
                }
                properties.write_to(&mut writer)?;

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
                properties,
            } => {
                //fixed header
                let mut byte_1 = 0x30_u8;

                if *retain_flag {
                    //we must replace any existing retained message for this topic and store
                    //the app message.
                    byte_1 |= 1 << 0;
                }

                if *qos == 0x01 {
                    byte_1 |= 1 << 1;
                    byte_1 |= 0 << 2;
                } else if *qos != 0x00 && *qos != 0x01 {
                    //we should throw a DISCONNECT with reason code 0x9B(QoS not supported).
                    println!("invalid qos");
                }

                if *dup_flag {
                    byte_1 |= 1 << 3;
                }

                //Dup flag must be set to 0 for all QoS 0 messages.
                if *qos == 0x00 {
                    byte_1 |= 0 << 3;
                }

                writer.write_all(&[byte_1])?;

                //Remaining Length
                write_string(&mut writer, topic_name)?;

                //packet_id
                write_u16(&mut writer, packet_id)?;

                //Properties
                properties.write_properties(&mut writer)?;

                //Payload
                write_string(&mut writer, payload)?;

                writer.flush()?;
                Ok(())
            }
        }
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<ClientMessage, Error> {
        let mut header = [0u8; 1];
        stream.read_exact(&mut header)?;

        let mut header = u8::from_le_bytes(header);
        let (mut dup_flag, mut qos, mut retain_flag) = (false, 0, false);

        let first_header_digits = header >> 4;
        if first_header_digits == 0x3 {
            let mask = 0b00001111;
            let last_header_digits = header & mask;

            header = 0x30_u8.to_le();

            if last_header_digits & 0b00000001 == 0b00000001 {
                retain_flag = true;
            }
            if last_header_digits & 0b00000010 == 0b00000010 {
                qos = 1;
            }
            if (last_header_digits >> 3) == 1 {
                dup_flag = true;
            }
        }

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
                println!("keep alive: {:?}", keep_alive);
                //properties
                //payload
                //client ID
                let client_id = read_string(stream)?;
                println!("client_id: {:?}", client_id);
                let will_properties = WillProperties::read_from(stream)?;
                println!("will properties: {:?}", will_properties);

                let mut last_will_topic = String::new();
                let mut will_message = String::new();
                if last_will_flag {
                    last_will_topic = read_string(stream)?;
                    will_message = read_string(stream)?;
                }
                print!("last will topic: {:?}", last_will_topic);

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

                let properties: ConnectProperties = ConnectProperties::read_from(stream)?;
                Ok(ClientMessage::Connect {
                    client_id: client_id.to_string(),
                    clean_start,
                    last_will_flag,
                    last_will_qos,
                    last_will_retain,
                    keep_alive,
                    properties,
                    will_properties,
                    last_will_topic: last_will_topic.to_string(),
                    last_will_message: will_message.to_string(),
                    username: user.to_string(),
                    password: pass.to_string(),
                })
            }
            0x30 => {
                let topic_properties = TopicProperties { topic_alias: 10, response_topic: "String".to_string() };
                let properties = PublishProperties::new(
                    1,
                    10,
                    topic_properties,
                    [1, 2, 3].to_vec(),
                    "a".to_string(),
                    1,
                    "a".to_string(),
                );
                let topic_name = read_string(stream)?;
                let packet_id = read_u16(stream)?;
                properties.read_properties(stream)?;
                let message = read_string(stream)?;
                Ok(ClientMessage::Publish {
                    packet_id,
                    topic_name,
                    qos,
                    retain_flag,
                    payload: message,
                    dup_flag,
                    properties,
                })
            }
            _ => Err(Error::new(std::io::ErrorKind::Other, "Invalid header")),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_publish_message_ok() {
        let topic_properties = TopicProperties { topic_alias: 10, response_topic: "String".to_string() };

        let properties = PublishProperties::new(
            1,
            10,
            topic_properties,
            [1, 2, 3].to_vec(),
            "a".to_string(),
            1,
            "a".to_string(),
        );

        let publish = ClientMessage::Publish {
            packet_id: 1,
            topic_name: "mensajes para juan".to_string(),
            qos: 1,
            retain_flag: true,
            payload: "hola soy juan".to_string(),
            dup_flag: true,
            properties,
        };

        let mut cursor = Cursor::new(Vec::<u8>::new());
        publish.write_to(&mut cursor).unwrap();
        cursor.set_position(0);

        match ClientMessage::read_from(&mut cursor) {
            Ok(read_publish) => {
                assert_eq!(publish, read_publish);
            }
            Err(e) => {
                panic!("no se pudo leer del cursor {:?}", e);
            }
        }
    }
}
