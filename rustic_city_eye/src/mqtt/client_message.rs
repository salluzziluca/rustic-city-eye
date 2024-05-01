use std::{
    fs::read, io::{BufWriter, Error, Read, Write}, net::TcpStream
};

//use self::quality_of_service::QualityOfService;

#[path = "quality_of_service.rs"]
mod quality_of_service;

#[derive(Debug)]
pub enum ClientMessage {
    Connect {
        //client_id: u32,
        // clean_session: bool,
        // username: String,
        // password: String,
        // lastWillTopic: String,
        // lastWillQoS: u8,
        // lasWillMessage: String,
        // lastWillRetain: bool,
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

fn write_string(stream: &mut dyn Write, string: &str) -> Result<(), Error> {
    let length = string.len() as u16;
    let length_bytes = length.to_be_bytes();
    stream.write(&length_bytes)?;
    stream.write(string.as_bytes())?;
    Ok(())
}

fn read_u16(stream: &mut dyn Read) -> Result<u16, Error> {
    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

fn read_string(stream: &mut dyn Read)-> Result<String, Error>{
    let string_length = read_u16(stream)?;
    let mut string_buf = vec![0; string_length as usize];
    stream.read_exact(&mut string_buf)?;

    let protocol_name =
        std::str::from_utf8(&string_buf).expect("Error al leer protocol_name");
    Ok(protocol_name.to_string())
}

impl ClientMessage {
    pub fn write_to(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let mut writer = BufWriter::new(stream);
        match self {
            ClientMessage::Connect {
                //client_id,
                // clean_session,
                // username,
                // password,
                // lastWillTopic,
                // lastWillQoS,
                // lasWillMessage,
                // lastWillRetain,
                // keepAlive,
            } => {
                //fixed header
                let byte_1: u8 = 0x10_u8.to_le();//00010000

                writer.write(&[byte_1])?;
                writer.flush()?;

                //protocol name
                let protocol_name = "MQTT";
                let protocol_name_length = protocol_name.len()  as u16;
                let protocol_name_length_bytes = protocol_name_length.to_le_bytes();
                writer.write(&[protocol_name_length_bytes[0]])?;
                writer.write(&[protocol_name_length_bytes[1]])?;
                writer.write(&protocol_name.as_bytes())?;


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
                //fixed header
                let mut byte_1 = 0x30_u8;

                if *retain_flag {
                    //we must replace any existing retained message for this topic and store
                    //the app message.
                    byte_1 |= 1 << 0;
                }

                if *qos == 0x01 {
                    byte_1 += 0x02_u8;
                } else if *qos == 0x03 || *qos == 0x02 {
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

                writer.write(&[byte_1])?;
                

                //Remaining Length 
                write_string(&mut writer, &topic_name)?;
                
                //todo: packet_id
                
                //Properties
                
                //Payload
                write_string(&mut writer, &payload)?;

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
                Ok(ClientMessage::Connect {})
            },
            0x30 => {
                let topic_name = read_string(stream)?;
                let message = read_string(stream)?;

                Ok(ClientMessage::Publish {
                    packet_id: 1,
                    topic_name,
                    qos: 0,
                    retain_flag: false,
                    payload: message,
                    dup_flag: false,
                })
            },
            0x31 => {
                let mut num_buffer = [0u8; 4];
                stream.read_exact(&mut num_buffer)?;
                // Una vez que leemos los bytes, los convertimos a un u32
                let size = u32::from_be_bytes(num_buffer);
                // Creamos un buffer para el nombre
                let mut topic_buf = vec![0; size as usize];
                stream.read_exact(&mut topic_buf)?;
                // Convierto de bytes a string.
                let topic_str = std::str::from_utf8(&topic_buf).expect("Error al leer topic");
                let topic_name = topic_str.to_owned();
                println!("{}", topic_name);

                Ok(ClientMessage::Publish {
                    packet_id: 1,
                    topic_name,
                    qos: 0,
                    retain_flag: true,
                    payload: "juancito".to_string(),
                    dup_flag: false,
                })
            },
            0x32 => {
                let mut num_buffer = [0u8; 4];
                stream.read_exact(&mut num_buffer)?;
                // Una vez que leemos los bytes, los convertimos a un u32
                let size = u32::from_be_bytes(num_buffer);
                // Creamos un buffer para el nombre
                let mut topic_buf = vec![0; size as usize];
                stream.read_exact(&mut topic_buf)?;
                // Convierto de bytes a string.
                let topic_str = std::str::from_utf8(&topic_buf).expect("Error al leer topic");
                let topic_name = topic_str.to_owned();
                println!("{}", topic_name);

                Ok(ClientMessage::Publish {
                    packet_id: 1,
                    topic_name,
                    qos: 1,
                    retain_flag: false,
                    payload: "juancito".to_string(),
                    dup_flag: false,
                })
            },
            0x33 => {
                let mut num_buffer = [0u8; 4];
                stream.read_exact(&mut num_buffer)?;
                // Una vez que leemos los bytes, los convertimos a un u32
                let size = u32::from_be_bytes(num_buffer);
                // Creamos un buffer para el nombre
                let mut topic_buf = vec![0; size as usize];
                stream.read_exact(&mut topic_buf)?;
                // Convierto de bytes a string.
                let topic_str = std::str::from_utf8(&topic_buf).expect("Error al leer topic");
                let topic_name = topic_str.to_owned();
                println!("{}", topic_name);

                Ok(ClientMessage::Publish {
                    packet_id: 1,
                    topic_name,
                    qos: 1,
                    retain_flag: true,
                    payload: "juancito".to_string(),
                    dup_flag: false,
                })
            },
            0x34 => {
                let mut num_buffer = [0u8; 4];
                stream.read_exact(&mut num_buffer)?;
                // Una vez que leemos los bytes, los convertimos a un u32
                let size = u32::from_be_bytes(num_buffer);
                // Creamos un buffer para el nombre
                let mut topic_buf = vec![0; size as usize];
                stream.read_exact(&mut topic_buf)?;
                // Convierto de bytes a string.
                let topic_str = std::str::from_utf8(&topic_buf).expect("Error al leer topic");
                let topic_name = topic_str.to_owned();
                println!("{}", topic_name);

                Ok(ClientMessage::Publish {
                    packet_id: 1,
                    topic_name,
                    qos: 2,
                    retain_flag: false,
                    payload: "juancito".to_string(),
                    dup_flag: false,
                })
            },
            0x35 => {
                let mut num_buffer = [0u8; 4];
                stream.read_exact(&mut num_buffer)?;
                // Una vez que leemos los bytes, los convertimos a un u32
                let size = u32::from_be_bytes(num_buffer);
                // Creamos un buffer para el nombre
                let mut topic_buf = vec![0; size as usize];
                stream.read_exact(&mut topic_buf)?;
                // Convierto de bytes a string.
                let topic_str = std::str::from_utf8(&topic_buf).expect("Error al leer topic");
                let topic_name = topic_str.to_owned();
                println!("{}", topic_name);

                Ok(ClientMessage::Publish {
                    packet_id: 1,
                    topic_name,
                    qos: 2,
                    retain_flag: true,
                    payload: "juancito".to_string(),
                    dup_flag: false,
                })
            },
            0x38 => {
                let mut num_buffer = [0u8; 4];
                stream.read_exact(&mut num_buffer)?;
                // Una vez que leemos los bytes, los convertimos a un u32
                let size = u32::from_be_bytes(num_buffer);
                // Creamos un buffer para el nombre
                let mut topic_buf = vec![0; size as usize];
                stream.read_exact(&mut topic_buf)?;
                // Convierto de bytes a string.
                let topic_str = std::str::from_utf8(&topic_buf).expect("Error al leer topic");
                let topic_name = topic_str.to_owned();
                println!("{}", topic_name);

                Ok(ClientMessage::Publish {
                    packet_id: 1,
                    topic_name,
                    qos: 0,
                    retain_flag: false,
                    payload: "juancito".to_string(),
                    dup_flag: true,
                })
            },
            0x39 => {
                let mut num_buffer = [0u8; 4];
                stream.read_exact(&mut num_buffer)?;
                // Una vez que leemos los bytes, los convertimos a un u32
                let size = u32::from_be_bytes(num_buffer);
                // Creamos un buffer para el nombre
                let mut topic_buf = vec![0; size as usize];
                stream.read_exact(&mut topic_buf)?;
                // Convierto de bytes a string.
                let topic_str = std::str::from_utf8(&topic_buf).expect("Error al leer topic");
                let topic_name = topic_str.to_owned();
                println!("{}", topic_name);

                Ok(ClientMessage::Publish {
                    packet_id: 1,
                    topic_name,
                    qos: 0,
                    retain_flag: true,
                    payload: "juancito".to_string(),
                    dup_flag: true,
                })
            },
            _ => Err(Error::new(std::io::ErrorKind::Other, "Invalid header")),
        }
    }
}