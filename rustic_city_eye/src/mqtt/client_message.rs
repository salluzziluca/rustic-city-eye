use std::io::{BufWriter, Error, Read, Write};


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
///Recibe un string y el stream al que escribir ese stream
/// 
/// Calcula su largo y luego escribe el largo y el string en el stream 
fn write_string(stream: &mut dyn Write, string: &str) -> Result<(), Error> {
    let length = string.len() as u16;
    let length_bytes = length.to_be_bytes();
    stream.write(&length_bytes)?;
    stream.write(string.as_bytes())?;
    Ok(())
}

fn write_u16(stream: &mut dyn Write, value: &u16) -> Result<(), Error> {
    let value_bytes = value.to_be_bytes();
    stream.write(&value_bytes)?;
    Ok(())
}

fn write_u32(stream: &mut dyn Write, value: &u32) -> Result<(), Error> {
    let value_bytes = value.to_be_bytes();
    stream.write(&value_bytes)?;
    Ok(())
}

fn read_string(stream: &mut dyn Read)-> Result<String, Error>{
    let string_length = read_u16(stream)?;
    let mut string_buf = vec![0; string_length as usize];
    stream.read_exact(&mut string_buf)?;

    let protocol_name =
        std::str::from_utf8(&string_buf).expect("Error al leer protocol_name");
    Ok(protocol_name.to_string())
}

fn read_u8(stream: &mut dyn Read) -> Result<u8, Error> {
    let mut buf = [0u8; 1];
    stream.read_exact(&mut buf)?;
    Ok(u8::from_be_bytes(buf))
}

fn read_u16(stream: &mut dyn Read) -> Result<u16, Error> {
    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

fn read_u32(stream: &mut dyn Read) -> Result<u32, Error> {
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
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
                write_u32(&mut writer, &last_will_delay_interval)?; 

                // let payload_format_indicator:u8;
                // match std::str::from_utf8(last_will_message) {
                //     Ok(_) => payload_format_indicator = 0x01,
                //     Err(_) => payload_format_indicator = 0x00,
                // }

                write_u16(&mut writer, message_expiry_interval)?;
                write_string(&mut writer, &content_type)?;

                //user property
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
                let will_delay_interval = read_u32(stream)?;

                
                let message_expiry_interval = read_u16(stream)?;

                let content_type = read_string(stream)?;

                //user property
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
