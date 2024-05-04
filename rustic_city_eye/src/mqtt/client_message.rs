use crate::mqtt::writer::*;
use crate::mqtt::reader::*;


use crate::mqtt::subscribe_properties::SubscribeProperties;
use std::any::TypeId;
use std::io::{BufWriter, Error, Read, Write};
use std::ops::Sub;


//use self::quality_of_service::QualityOfService;
const PROTOCOL_VERSION: u8 = 5;
const WILL_DELAY_INTERVAL_ID: u8 = 0x18;
const PAYLOAD_FORMAT_INDICATOR_ID: u8 = 0x01;
const MESSAGE_EXPIRY_INTERVAL_ID: u8 = 0x02;
const CONTENT_TYPE_ID: u8 = 0x03;
const RESPONSE_TOPIC_ID: u8 = 0x08;
const CORRELATION_DATA_ID: u8 = 0x09;
const USER_PROPERTY_ID: u8 = 0x26;

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
    /// last_will_delay_interval especifica el tiempo en segundos que el broker debe esperar antes de publicar el will message.
    /// message_expiry_interval especifica el tiempo en segundos que el broker debe esperar antes de descartar el will message.
    /// content_type especifica el tipo de contenido del will message. (ej json, plain text)
    /// 
    /// user_property especifica una propiedad del usuario que se envia en el mensaje, se pueden enviar 0, 1 o más propiedades.
    Connect {
        client_id: String,
        clean_start: bool,
        last_will_flag: bool, 
        last_will_qos: u8,    
        last_will_retain: bool, 
        username: String,
        password: String,
        keep_alive: u16,
        last_will_delay_interval: u32,
        //message_expiry_interval: u16,
        content_type: String,
        response_topic: String,
        correlation_data: Vec<u8>,
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
    /// El Subscribe Message se utiliza para suscribirse a uno o más topics. El cliente puede enviar un mensaje de subscribe con un packet id y una lista de topics a los que se quiere suscribir. El broker responde con un mensaje de suback con el mismo packet id y una lista de return codes que indican si la suscripcion fue exitosa o no.
    /// 
    /// packet_id es un identificador unico para el mensaje de subscribe.
    /// topic_name es el nombre del topic al que se quiere suscribir.
    /// properties es un struct que contiene las propiedades del mensaje de subscribe.
    ///
    Subscribe {
        /// packet_id es un identificador unico para el mensaje de subscribe.
        packet_id: u16,
        /// topic_name es el nombre del topic al que se quiere suscribir.
        topic_name: String,
        /// properties es un struct que contiene las propiedades del mensaje de subscribe.
        properties: SubscribeProperties,
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
                username,
                password,
                keep_alive,
                last_will_delay_interval,
                //message_expiry_interval,
                content_type,
                user_property,
                last_will_message,
                response_topic,
                correlation_data,
    
                // lastWillTopic,
                // last_will_qos,
                // lasWillMessage,
                // last_will_retain,
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
                // if connect_flags > 3 {
                //     Err(std::io::Error::new("QoS cant be greater than 3"))
                // }
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
                //let mut property_length = 0;
                //TODO: implementar las 300 millones de propiedades, por ahi estría bueno usar un struct

                //payload
                //client ID
                write_string(&mut writer, &client_id)?;
     

                //will properties
                write_u8(&mut writer, &WILL_DELAY_INTERVAL_ID)?;

                write_u32(&mut writer, &last_will_delay_interval)?; 

                let payload_format_indicator:u8 = 0x01; //siempre 1 porque los strings en rust siempre son utf-8
                write_u8(&mut writer, &PAYLOAD_FORMAT_INDICATOR_ID)?;
                write_u8(&mut writer, &payload_format_indicator)?;

                // write_u8(&mut writer, &MESSAGE_EXPIRY_INTERVAL_ID)?;
                // write_u16(&mut writer, message_expiry_interval)?;

                write_u8(&mut writer, &CONTENT_TYPE_ID)?;
                write_string(&mut writer, &content_type)?;

                write_u8(&mut writer, &RESPONSE_TOPIC_ID)?;
                write_string(&mut writer, &response_topic)?;

                write_u8(&mut writer, &CORRELATION_DATA_ID)?;
                let correlation_data_length = correlation_data.len() as u16;
                write_u16(&mut writer, &correlation_data_length)?;
                for byte in correlation_data {
                    write_u8(&mut writer, byte)?;
                }
                //user property

                write_u8(&mut writer, &USER_PROPERTY_ID)?;
                if let Some((key, value)) = user_property {
                    write_string(&mut writer, &key)?;

                    write_string(&mut writer, &value)?;
                }

                //will payload

                write_string(&mut writer, &last_will_message)?;

                //username

                if username.len() != 0 {
                    write_string(&mut writer, &username)?;
                }

                //password
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
            },
            ClientMessage::Subscribe {
                packet_id,
                topic_name,
                properties,
            } => {
                let byte_1: u8 = 0x82_u8;
                writer.write(&[byte_1])?;
                write_u16(&mut writer, &packet_id)?;
                write_string(&mut writer, &topic_name)?;
                properties.write_properties(&mut writer)?;
                writer.flush()?;
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
                let  protocol_name = read_string(stream)?;

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
                println!("protocol version: {:?}", protocol_version);

                //connect flags
                let connect_flags = read_u8(stream)?;



                //keep alive
                let keep_alive = read_u16(stream)?;

                // connect properties


                //payload
                //client ID
                let client_id = read_string(stream)?;

                //will properties
                let will_delay_interval_id = read_u8(stream)?;
                if will_delay_interval_id != WILL_DELAY_INTERVAL_ID {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid will delay interval id",
                    ));
                }
                let will_delay_interval = read_u32(stream)?;

                let payload_format_indicator_id = read_u8(stream)?;
                if payload_format_indicator_id != PAYLOAD_FORMAT_INDICATOR_ID {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid payload format indicator id",
                    ));
                }
                let message_expiry_interval_id = read_u8(stream)?;

                // if message_expiry_interval_id != MESSAGE_EXPIRY_INTERVAL_ID {
                //     return Err(Error::new(
                //         std::io::ErrorKind::Other,
                //         "Invalid message expiry interval id",
                //     ));
                // }
                // let message_expiry_interval = read_u16(stream)?;

                let content_type_id = read_u8(stream)?;
                if content_type_id != CONTENT_TYPE_ID {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid content type id",
                    ));
                }
                let content_type = read_string(stream)?;

                let response_topic_id = read_u8(stream)?;
                if response_topic_id != RESPONSE_TOPIC_ID {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid response topic id",
                    ));
                }
                let response_topic = read_string(stream)?;

                let correlation_data_id = read_u8(stream)?;
                if correlation_data_id != CORRELATION_DATA_ID {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid correlation data id",
                    ));
                }
                let correlation_data_length = read_u16(stream)?;
                let mut correlation_data = vec![0; correlation_data_length as usize];
                stream.read_exact(&mut correlation_data)?;

                //user property

                let user_property_id = read_u8(stream)?;
                if user_property_id != USER_PROPERTY_ID {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid user property id",
                    ));
                }
                let user_property_key = read_string(stream)?;

                let user_property_value = read_string(stream)?;

                //will payload
                let will_message = read_string(stream)?;

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
                    clean_start: (connect_flags & (1 << 1)) != 0,
                    last_will_flag: (connect_flags & (1 << 2)) != 0,
                    last_will_qos: (connect_flags >> 3) & 0b11,
                    last_will_retain: (connect_flags & (1 << 5)) != 0,
                    username: user.to_string(),
                    password: pass.to_string(),
                    keep_alive: keep_alive,
                    client_id: client_id.to_string(),
                    last_will_delay_interval: will_delay_interval,
                   // message_expiry_interval: message_expiry_interval,
                    content_type: content_type.to_string(),
                    user_property: Some((user_property_key.to_string(), user_property_value.to_string())),
                    last_will_message: will_message.to_string(),
                    response_topic: response_topic.to_string(),
                    correlation_data: correlation_data,
            
                    
                })
            },
            0x82 => {
                let packet_id = read_u16(stream)?;
                let topic = read_string(stream)?;
                let properties = SubscribeProperties::read_properties(stream)?;
                Ok(ClientMessage::Subscribe {
                    packet_id: packet_id,
                    topic_name: topic,
                    properties: properties,
                })
            },
            _ => Err(Error::new(std::io::ErrorKind::Other, "Invalid header")),
        }
    }
}
