use crate::mqtt::subscribe_properties::SubscribeProperties;
use std::io::{BufWriter, Error, Read, Write};

use crate::mqtt::connack_properties::ConnackProperties;
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

    // EL connack el paquete de respuesta que el broker envia al cliente despues de recibir un connect message.
    Connack {
        session_present: bool,
        reason_code: u8,
        properties: ConnackProperties,
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
            ClientMessage::Subscribe {
                packet_id,
                topic_name,
                properties,
            } => {
                let byte_1: u8 = 0x82_u8;
                writer.write_all(&[byte_1])?;
                write_u16(&mut writer, packet_id)?;
                write_string(&mut writer, topic_name)?;
                properties.write_properties(&mut writer)?;
                writer.flush()?;
                Ok(())
            }
            ClientMessage::Connack {
                session_present,
                reason_code,
                properties,
            } => {
                let byte_1: u8 = 0x20_u8; //00100000
                writer.write_all(&[byte_1])?;
                println!("session present: {:?}", session_present);
                writer.write_all(&[if *session_present { 1u8 } else { 0u8 }])?;
                writer.write_all(&[*reason_code])?;
                properties.write_to(&mut writer)?;
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
            0x20 => {
                let session_present = read_u8(stream)? == 1;
                let reason_code = read_u8(stream)?;
                let properties = ConnackProperties::read_from(stream)?;
                Ok(ClientMessage::Connack {
                    session_present,
                    reason_code,
                    properties,
                })
            }
            _ => Err(Error::new(std::io::ErrorKind::Other, "Invalid header")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mqtt::broker_message::BrokerMessage;
    use crate::mqtt::connack_properties::ConnackPropertiesBuilder;
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_publish_message_ok() {
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

    #[test]
    fn test_client_message() {
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
        connect.write_to(&mut cursor).unwrap();
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
    fn test_sin_props() {
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
        connect.write_to(&mut cursor).unwrap();
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
    fn test_connack() {
        let properties = ConnackPropertiesBuilder::new()
            .session_expiry_interval(100)
            .receive_maximum(10)
            .maximum_qos(true)
            .retain_available(true)
            .maximum_packet_size(100)
            .assigned_client_identifier("client_id".to_owned())
            .topic_alias_maximum(10)
            .reason_string("reason".to_owned())
            .user_properties(vec![("property".to_string(), "value".to_string())])
            .wildcard_subscription_available(true)
            .subscription_identifier_available(true)
            .shared_subscription_available(true)
            .server_keep_alive(10)
            .response_information("Testing".to_owned())
            .server_reference("server".to_owned())
            .authentication_method("auth".to_owned())
            .authentication_data("data".to_owned().as_bytes().to_vec()) // Convert string to Vec<u8>
            .build()
            .unwrap();

        println!("{:?}", properties);
        let connack = ClientMessage::Connack {
            session_present: true,
            reason_code: 0,
            properties,
        };

        let mut cursor = Cursor::new(Vec::<u8>::new());
        connack.write_to(&mut cursor).unwrap();
        cursor.set_position(0);

        match ClientMessage::read_from(&mut cursor) {
            Ok(read_connack) => {
                assert_eq!(connack, read_connack);
            }
            Err(e) => {
                panic!("Failed to read from cursor: {:?}", e);
            }
        }
    }

    #[test]
    fn test_sub() {
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
        sub.write_to(&mut cursor).unwrap();
        cursor.set_position(0);
        let read_sub = ClientMessage::read_from(&mut cursor).unwrap();
        assert_eq!(sub, read_sub);
    }

    #[test]
    fn test_suback() {
        let suback = BrokerMessage::Suback {
            reason_code: 1,
            packet_id_msb: 1,
            packet_id_lsb: 1,
        };

        let mut cursor = Cursor::new(Vec::<u8>::new());
        suback.write_to(&mut cursor).unwrap();
        cursor.set_position(0);
        let read_suback = BrokerMessage::read_from(&mut cursor).unwrap();
        assert_eq!(suback, read_suback);
    }
}
