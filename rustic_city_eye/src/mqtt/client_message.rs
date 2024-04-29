use std::{
    io::{BufWriter, Error, Read, Write},
    net::TcpStream,
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
                //byte 1 process
                let mut byte_1_value = 0x30_u8;

                if *retain_flag {
                    byte_1_value += 0x01_u8;
                }

                if *qos == 0x01 {
                    byte_1_value += 0x02_u8;
                } else if *qos == 0x02 {
                    byte_1_value += 0x04_u8;
                } else if *qos == 0x03 {
                    println!("qos invalida");
                }

                if *dup_flag {
                    byte_1_value += 0x08_u8;
                }

                let byte_1: u8 = byte_1_value.to_le();

                writer.write(&[byte_1])?;
                writer.flush()?;

                //remaining length process
                let size_be = (topic_name.len() as u32).to_be_bytes();
                writer.write(&size_be)?;
                writer.write(&topic_name.as_bytes())?;

                let size_be = (payload.len() as u32).to_be_bytes();
                writer.write(&size_be)?;
                writer.write(&payload.as_bytes())?;

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
                
                stream.read_exact(&mut num_buffer)?;
                let size = u32::from_be_bytes(num_buffer);
                // // Creamos un buffer para el nombre
                let mut message_buf = vec![0; size as usize];
                stream.read_exact(&mut message_buf)?;
                // // Convierto de bytes a string.
                let message_str = std::str::from_utf8(&message_buf).expect("Error al leer mensaje");
                let message = message_str.to_owned();
                

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