use crate::mqtt::subscribe_properties::SubscribeProperties;
use std::io::{BufWriter, Error, Read, Write};

use crate::mqtt::connect_properties::ConnectProperties;
use crate::mqtt::publish_properties::{PublishProperties, TopicProperties};
use crate::mqtt::reader::*;
use crate::mqtt::will_properties::*;
use crate::mqtt::writer::*;

//use self::quality_of_service::QualityOfService;
const PROTOCOL_VERSION: u8 = 5;
const SESSION_EXPIRY_INTERVAL_ID: u8 = 0x11;
const REASON_STRING_ID: u8 = 0x1F;
const USER_PROPERTY_ID: u8 = 0x26;

#[derive(Debug, PartialEq, Clone)]
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

    /// El paquete Publish es enviado desde un cliente al servidor, o desde un servidor al cliente para transportar un mensaje de aplicacion.
    Publish {
        packet_id: u16,

        ///Identifica el canal de informacion por el cual el Payload data se va a publicar.
        topic_name: String,

        ///Indica el nivel de garantia de delivery de un application message.
        qos: usize,

        /// Si vale 1, el server debe reemplazar cualquier retained message para ese topic
        /// y guardar este application message.
        retain_flag: usize,

        ///Es el application message que se esta publicando.
        ///El contenido y formato de la data es especificado por la aplicacion.
        payload: String,

        ///Dup flag indica si fue la primera ocasion que el client o el servidor intento enviar este packete.
        /// Debe ser 0 para todos los mensajes con QoS 0.
        /// Debe valer 1 para indicar que es el segundo intento de envio del packete.
        dup_flag: usize,

        /// Una property es un identificador que define el uso y tipos de data, seguido de un valor.
        properties: PublishProperties,
    },

    /// El Subscribe Message se utiliza para suscribirse a uno o más topics. El cliente puede enviar un mensaje de subscribe con un packet id y una lista de topics a los que se quiere suscribir. El broker responde con un mensaje de suback con el mismo packet id y una lista de return codes que indican si la suscripcion fue exitosa o no.
    Subscribe {
        packet_id: u16,
        /// topic_name es el nombre del topic al que se quiere suscribir.
        topic_name: String,
        /// properties es un struct que contiene las propiedades del mensaje de subscribe.
        properties: SubscribeProperties,
    },
    Unsubscribe {
        packet_id: u16,
        topic_name: String,
        properties: SubscribeProperties,
    },
    /// Es el ultimo mensaje que el cliente envia antes de desconectarse, este mensaje contiene informacion sobre la razon de la desconexión y propiedades adicionales.
    /// reason_code es el codigo de la razon de la desconexión.
    /// session_expiry_interval es el tiempo en segundos que el broker debe mantener la sesion del cliente activa despues de que este se desconecte.
    /// reason_string es un mensaje de texto que describe la razon de la desconexión.
    /// user_properties es un conjunto de propiedades adicionales que el cliente puede enviar.
    Disconnect {
        reason_code: u8,
        session_expiry_interval: u32,
        reason_string: String,
        user_properties: Vec<(String, String)>,
    },
    Pingreq,
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
                self.write_first_packet_byte(&mut writer)?;

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
                packet_id: _,
                topic_name: _,
                qos: _,
                retain_flag: _,
                payload,
                dup_flag: _,
                properties: _,
            } => {
                //fixed header
                self.write_first_packet_byte(&mut writer)?;

                //Remaining Length
                self.write_packet_properties(&mut writer)?;

                //Payload
                write_string(&mut writer, payload)?;

                writer.flush()?;
                Ok(())
            }
            ClientMessage::Subscribe {
                packet_id: _,
                topic_name: _,
                properties: _,
            } => {
                self.write_first_packet_byte(&mut writer)?;
                self.write_packet_properties(&mut writer)?;
                writer.flush()?;
                Ok(())
            }
            ClientMessage::Unsubscribe {
                packet_id: _,
                topic_name: _,
                properties: _,
            } => {
                self.write_first_packet_byte(&mut writer)?;
                self.write_packet_properties(&mut writer)?;
                writer.flush()?;
                Ok(())
            }
            ClientMessage::Disconnect {
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
            ClientMessage::Pingreq => {
                let byte_1: u8 = 0xC0_u8;
                writer.write_all(&[byte_1])?;
                writer.flush()?;
                Ok(())
            }
        }
    }

    fn write_first_packet_byte(
        &self,
        writer: &mut BufWriter<&mut dyn Write>,
    ) -> std::io::Result<()> {
        match self {
            ClientMessage::Connect {
                clean_start: _,
                last_will_flag: _,
                last_will_qos: _,
                last_will_retain: _,
                keep_alive: _,
                properties: _,
                client_id: _,
                will_properties: _,
                last_will_topic: _,
                last_will_message: _,
                username: _,
                password: _,
            } => {
                let byte_1: u8 = 0x10_u8.to_le(); //00010000
                writer.write_all(&[byte_1])?;
            }
            ClientMessage::Publish {
                packet_id: _,
                topic_name: _,
                qos,
                retain_flag,
                payload: _,
                dup_flag,
                properties: _,
            } => {
                let mut byte_1 = 0x30_u8;

                if *retain_flag == 1 {
                    //we must replace any existing retained message for this topic and store
                    //the app message.
                    byte_1 |= 1 << 0;
                }

                if *qos == 1 {
                    byte_1 |= 1 << 1;
                    byte_1 |= 0 << 2;
                } else if *qos != 0x00 && *qos != 0x01 {
                    //we should throw a DISCONNECT with reason code 0x81(Malformed packet).
                    println!("Qos inválido");
                }

                if *dup_flag == 1 {
                    byte_1 |= 1 << 3;
                }

                //Dup flag must be set to 0 for all QoS 0 messages.
                if *qos == 0x00 {
                    byte_1 |= 0 << 3;
                }

                writer.write_all(&[byte_1])?;
            }
            ClientMessage::Subscribe {
                packet_id: _,
                topic_name: _,
                properties: _,
            } => {
                let byte_1: u8 = 0x82_u8;
                writer.write_all(&[byte_1])?;
            }
            ClientMessage::Unsubscribe {
                packet_id: _,
                topic_name: _,
                properties: _,
            } => {
                let byte_1: u8 = 0xA2_u8;
                writer.write_all(&[byte_1])?;
            }
            ClientMessage::Disconnect {
                reason_code: _,
                session_expiry_interval: _,
                reason_string: _,
                user_properties: _,
            } => {
                let byte_1: u8 = 0xE0_u8;
                writer.write_all(&[byte_1])?;
            }
            ClientMessage::Pingreq => {
                let byte_1: u8 = 0xC0_u8;
                writer.write_all(&[byte_1])?;
            }
        }
        Ok(())
    }

    fn write_packet_properties(
        &self,
        writer: &mut BufWriter<&mut dyn Write>,
    ) -> std::io::Result<()> {
        match self {
            ClientMessage::Connect {
                clean_start: _,
                last_will_flag: _,
                last_will_qos: _,
                last_will_retain: _,
                keep_alive: _,
                properties: _,
                client_id: _,
                will_properties: _,
                last_will_topic: _,
                last_will_message: _,
                username: _,
                password: _,
            } => todo!(),
            ClientMessage::Publish {
                packet_id,
                topic_name,
                qos: _,
                retain_flag: _,
                payload: _,
                dup_flag: _,
                properties,
            } => {
                write_u16(writer, packet_id)?;

                write_string(writer, topic_name)?;

                //Properties
                properties.write_properties(writer)?;
            }
            ClientMessage::Subscribe {
                packet_id,
                topic_name,
                properties,
            } => {
                write_u16(writer, packet_id)?;

                write_string(writer, topic_name)?;

                //Properties
                properties.write_properties(writer)?;
            }
            ClientMessage::Unsubscribe {
                packet_id,
                topic_name,
                properties,
            } => {
                write_u16(writer, packet_id)?;

                write_string(writer, topic_name)?;

                //Properties
                properties.write_properties(writer)?;
            },
           ClientMessage::Disconnect {
                reason_code,
                session_expiry_interval,
                reason_string,
                user_properties,
            } => {
                //fixed header
                let header: u8 = 0xE0_u8.to_le(); //11100000
                write_u8(writer, &header)?;
                //variable_header
                write_u8(writer, reason_code)?;

                write_u8(writer, &SESSION_EXPIRY_INTERVAL_ID)?;
                write_u32(writer, session_expiry_interval)?;

                write_u8(writer, &REASON_STRING_ID)?;
                write_string(writer, reason_string)?;

                write_u8( writer, &USER_PROPERTY_ID)?;
                write_string_pairs(writer, user_properties)?;
                writer.flush()?;
                
            }

            ClientMessage::Pingreq => {
                let byte_1: u8 = 0xC0_u8;
                writer.write_all(&[byte_1])?;
                writer.flush()?;
              
            }
        }

        Ok(())
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<ClientMessage, Error> {
        let mut header = [0u8; 1];
        stream.read_exact(&mut header)?;

        let mut header = u8::from_le_bytes(header);
        let (mut dup_flag, mut qos, mut retain_flag) = (0, 0, 0);

        let first_header_digits = header >> 4;
        if first_header_digits == 0x3 {
            let mask = 0b00001111;
            let last_header_digits = header & mask;

            header = 0x30_u8.to_le();

            if last_header_digits & 0b00000001 == 0b00000001 {
                retain_flag = 1;
            }
            if last_header_digits & 0b00000010 == 0b00000010 {
                qos = 1;
            }
            if (last_header_digits >> 3) == 1 {
                dup_flag = 1;
            }
        }

        match header {
            0x10 => {
                //leo el protocol name

                let protocol_name = read_string(stream)?;

                if protocol_name != "MQTT" {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Nombre de protocolo inválido",
                    ));
                }
                //protocol version
                let protocol_version = read_u8(stream)?;

                if protocol_version != PROTOCOL_VERSION {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Version de protocol inválido",
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
                let topic_properties = TopicProperties {
                    topic_alias: 10,
                    response_topic: "String".to_string(),
                };

                let properties = PublishProperties::new(
                    1,
                    10,
                    topic_properties,
                    [1, 2, 3].to_vec(),
                    "a".to_string(),
                    1,
                    "a".to_string(),
                );
                let packet_id = read_u16(stream)?;
                let topic_name = read_string(stream)?;
                PublishProperties::read_from(stream)?;
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
            0x82 => {
                let packet_id = read_u16(stream)?;
                let topic = read_string(stream)?;
                let properties = SubscribeProperties::read_properties(stream)?;
                Ok(ClientMessage::Subscribe {
                    packet_id,
                    topic_name: topic,
                    properties,
                })
            }
            0xA2 => {
                let packet_id = read_u16(stream)?;
                let topic = read_string(stream)?;
                let properties = SubscribeProperties::read_properties(stream)?;
                Ok(ClientMessage::Unsubscribe {
                    packet_id,
                    topic_name: topic,
                    properties,
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

                Ok(ClientMessage::Disconnect {
                    reason_code,
                    session_expiry_interval,
                    reason_string,
                    user_properties,
                })
            }
            0xC0 => Ok(ClientMessage::Pingreq),
            _ => Err(Error::new(std::io::ErrorKind::Other, "Invalid header")),
        }
    }

    // pub fn analize_packet_id(&self, packet: u16) -> bool {
    //     match self {
    //         ClientMessage::Connect { clean_start, last_will_flag, last_will_qos, last_will_retain, keep_alive, properties, client_id, will_properties, last_will_topic, last_will_message, username, password } => todo!(),
    //         ClientMessage::Publish { packet_id, topic_name, qos, retain_flag, payload, dup_flag, properties } => {
    //             if *packet_id == packet {
    //                 return true;
    //             }
    //             false
    //         },
    //         ClientMessage::Subscribe { packet_id, topic_name, properties } => {
    //             if *packet_id == packet {
    //                 return true;
    //             }
    //             false
    //         },
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_01_connect_message_ok() {
        let connect_propierties = ConnectProperties {
            session_expiry_interval: 1,
            receive_maximum: 2,
            maximum_packet_size: 10,
            topic_alias_maximum: 99,
            request_response_information: true,
            request_problem_information: false,
            user_properties: vec![
                ("Hola".to_string(), "Mundo".to_string()),
                ("Chau".to_string(), "Mundo".to_string()),
            ],
            authentication_method: "test".to_string(),
            authentication_data: vec![1_u8, 2_u8, 3_u8, 4_u8, 5_u8],
        };
        let will_properties = WillProperties::new(
            120,
            1,
            30,
            "plain".to_string(),
            "topic".to_string(),
            vec![1, 2, 3, 4, 5],
            vec![("propiedad".to_string(), "valor".to_string())],
        );
        let connect = ClientMessage::Connect {
            clean_start: true,
            last_will_flag: true,
            last_will_qos: 1,
            last_will_retain: true,
            keep_alive: 35,
            properties: connect_propierties,
            client_id: "kvtr33".to_string(),
            will_properties,
            last_will_topic: "topic".to_string(),
            last_will_message: "chauchis".to_string(),
            username: "prueba".to_string(),
            password: "".to_string(),
        };
        let mut cursor = Cursor::new(Vec::<u8>::new());
        match connect.write_to(&mut cursor) {
            Ok(_) => {}
            Err(e) => {
                panic!("no se pudo escribir en el cursor {:?}", e);
            }
        }

        cursor.set_position(0);

        match ClientMessage::read_from(&mut cursor) {
            Ok(read_connect) => {
                assert_eq!(connect, read_connect);
            }
            Err(e) => {
                panic!("no se pudo leer del cursor {:?}", e);
            }
        }
    }

    #[test]
    fn test_02_connect_without_props_err() {
        let connect_properties = ConnectProperties {
            session_expiry_interval: 0,
            receive_maximum: 0,
            maximum_packet_size: 0,
            topic_alias_maximum: 0,
            request_response_information: false,
            request_problem_information: false,
            user_properties: vec![],
            authentication_method: "".to_string(),
            authentication_data: vec![],
        };

        let connect = ClientMessage::Connect {
            clean_start: true,
            last_will_flag: true,
            last_will_qos: 1,
            last_will_retain: true,
            keep_alive: 35,
            properties: connect_properties,
            client_id: "kvtr33".to_string(),
            will_properties: WillProperties::new(
                0,
                1,
                0,
                "".to_string(),
                "".to_string(),
                vec![],
                vec![],
            ),
            last_will_topic: "topic".to_string(),
            last_will_message: "chauchis".to_string(),
            username: "prueba".to_string(),
            password: "".to_string(),
        };
        let mut cursor = Cursor::new(Vec::<u8>::new());
        match connect.write_to(&mut cursor) {
            Ok(_) => {}
            Err(e) => {
                panic!("no se pudo escribir en el cursor {:?}", e);
            }
        }
        cursor.set_position(0);

        match ClientMessage::read_from(&mut cursor) {
            Ok(read_connect) => {
                assert_eq!(connect, read_connect);
            }
            Err(e) => {
                panic!("no se pudo leer del cursor {:?}", e);
            }
        }
    }

    #[test]
    fn test_03_publish_message_ok() {
        let topic_properties = TopicProperties {
            topic_alias: 10,
            response_topic: "String".to_string(),
        };

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
            retain_flag: 1,
            payload: "hola soy juan".to_string(),
            dup_flag: 1,
            properties,
        };

        let mut cursor = Cursor::new(Vec::<u8>::new());
        match publish.write_to(&mut cursor) {
            Ok(_) => {}
            Err(e) => {
                panic!("no se pudo escribir en el cursor {:?}", e);
            }
        }
        cursor.set_position(0);

        match ClientMessage::read_from(&mut cursor) {
            Ok(read_publish) => {
                println!("{:?}", read_publish);
                assert_eq!(publish, read_publish);
            }
            Err(e) => {
                panic!("no se pudo leer del cursor {:?}", e);
            }
        }
    }

    #[test]
    fn test_04_subscribe_ok() {
        let sub = ClientMessage::Subscribe {
            packet_id: 1,
            topic_name: "topico".to_string(),
            properties: SubscribeProperties::new(
                1,
                vec![("propiedad".to_string(), "valor".to_string())],
                vec![0, 1, 2, 3],
            ),
        };

        let mut cursor = Cursor::new(Vec::<u8>::new());
        match sub.write_to(&mut cursor) {
            Ok(_) => {}
            Err(e) => {
                panic!("no se pudo escribir en el cursor {:?}", e);
            }
        }
        cursor.set_position(0);
        let read_sub = match ClientMessage::read_from(&mut cursor) {
            Ok(sub) => sub,
            Err(e) => {
                panic!("no se pudo leer del cursor {:?}", e);
            }
        };
        assert_eq!(sub, read_sub);
    }
    
    #[test]
    fn test_05_disconnect_ok() {
        let disconect = ClientMessage::Disconnect {
            reason_code: 1,
            session_expiry_interval: 1,
            reason_string: "hola".to_string(),
            user_properties: vec![("hola".to_string(), "mundo".to_string())],
        };

        let mut cursor = Cursor::new(Vec::<u8>::new());
        match disconect.write_to(&mut cursor) {
            Ok(_) => {}
            Err(e) => {
                panic!("no se pudo escribir en el cursor {:?}", e);
            }
        }
        cursor.set_position(0);
        let read_disconect = match ClientMessage::read_from(&mut cursor) {
            Ok(disconect) => disconect,
            Err(e) => {
                panic!("no se pudo leer del cursor {:?}", e);
            }
        };
        assert_eq!(disconect, read_disconect);
    }
}
