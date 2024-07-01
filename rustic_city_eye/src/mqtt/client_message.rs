use serde::{Deserialize, Serialize};

use crate::mqtt::subscribe_properties::SubscribeProperties;
use crate::utils::payload_types::PayloadTypes;
use std::fs::File;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Read, Write};

use crate::mqtt::publish::publish_properties::PublishProperties;
use crate::utils::{reader::*, writer::*};

use super::connect::connect_properties::ConnectProperties;
use super::connect::will_properties::WillProperties;
use super::messages_config::MessagesConfig;
use super::payload::Payload;
use super::subscription::Subscription;

use super::protocol_error::ProtocolError;
use crate::mqtt::connect::last_will::LastWill;
const PROTOCOL_VERSION: u8 = 5;
const SESSION_EXPIRY_INTERVAL_ID: u8 = 0x11;
const REASON_STRING_ID: u8 = 0x1F;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
///El Connect Message es el primer mensaje que el cliente envia cuando se conecta al broker. Este contiene toda la informacion necesaria para que el broker identifique al cliente y pueda establecer una sesion con los parametros establecidos.
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
pub struct Connect {
    clean_start: bool,
    last_will_flag: bool,
    last_will_qos: u8,
    last_will_retain: bool,
    keep_alive: u16,
    pub(crate) properties: ConnectProperties,

    /// Connect Payload
    /// Ayuda a que el servidor identifique al cliente. Siempre debe ser
    /// el primer campo del payload del packet Connect.
    pub(crate) client_id: String,

    will_properties: Option<WillProperties>,
    last_will_topic: Option<String>,
    last_will_message: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ClientMessage {
    Connect(Connect),

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
        payload: PayloadTypes,

        ///Dup flag indica si fue la primera ocasion que el client o el servidor intento enviar este packete.
        /// Debe ser 0 para todos los mensajes con QoS 0.
        /// Debe valer 1 para indicar que es el segundo intento de envio del packete.
        dup_flag: usize,

        /// Una property es un identificador que define el uso y tipos de data, seguido de un valor.
        properties: PublishProperties,
    },

    /// El Subscribe Message se utiliza para suscribirse a uno o más topics. El cliente puede enviar un mensaje de subscribe con un packet id y una lista de topics a los que se quiere suscribir. El broker responde con un mensaje de suback con el mismo packet id y una lista de return codes que indican si la suscripcion fue exitosa o no.
    Subscribe {
        /// packet_id es un identificador unico que el cliente asigna a cada mensaje que envia.
        packet_id: u16,
        /// properties es un struct que contiene las propiedades del mensaje de subscribe.
        properties: SubscribeProperties,
        /// Vector de subscription es un struct que contiene la informacion de la suscripcion.
        payload: Subscription,
    },
    /// El Unsubscribe Message se utiliza para cancelar una o más suscripciones. El cliente envia un mensaje de unsubscribe con un packet id y una lista de topics de los que quiere desuscribirse. El broker responde con un mensaje de unsuback con el mismo packet id y una lista de return codes que indican si la desuscripcion fue exitosa o no.
    Unsubscribe {
        /// packet_id es un identificador unico que el cliente asigna a cada mensaje que envia.
        packet_id: u16,
        /// properties es un struct que contiene las propiedades del mensaje de unsubscribe.
        properties: SubscribeProperties,
        /// Vector de subscription es un struct que contiene la informacion de la suscripcion.
        payload: Subscription,
    },
    /// Es el ultimo mensaje que el cliente envia antes de desconectarse, este mensaje contiene informacion sobre la razon de la desconexión y propiedades adicionales.
    Disconnect {
        /// reason_code es el codigo de la razon de la desconexión.
        reason_code: u8,
        /// session_expiry_interval es el tiempo en segundos que el broker debe mantener la sesion del cliente activa despues de que este se desconecte.
        session_expiry_interval: u32,
        /// reason_string es un mensaje de texto que describe la razon de la desconexión.
        reason_string: String,
        /// client_id es el identificador unico del cliente.
        client_id: String,
    },
    /// El Pingreq Message es un mensaje que el cliente envia al broker para mantener la conexion activa.
    Pingreq,

    /// Sirve para autenticar usuarios. Tanto el Broker como el Client pueden enviar estos packets(van a ser iguales).
    ///
    /// La idea es utilizar propiedades que se definen dentro de los packets del tipo Connect, y poder realizar la
    /// autenticacion correctamente.
    Auth {
        /// Nos indica el estado de nuestra autenticacion.
        reason_code: u8,

        /// Indica el metodo de autenticacion a seguir.
        authentication_method: String,

        /// Contiene data binaria sobre la autenticacion.
        authentication_data: Vec<u8>,

        /// Aca se muestra mas a detalle la razon de la desconexion. La idea es mostrarle
        /// al usuario a traves de un texto legible el por que el broker decidio desconectarlo.
        reason_string: String,

        /// Para diagnosticos e informacion adicionales.
        user_properties: Vec<(String, String)>,
    },
}
impl MessagesConfig for Connect {
    /// Hereda del trait de MessagesConfig, por lo que sabe
    /// crear un ClientMessage -> en este caso devolvera
    /// siempre un mensaje del tipo Connect.
    fn parse_message(&self, _packet_id: u16) -> ClientMessage {
        ClientMessage::Connect(self.clone())
    }
}
impl MessagesConfig for ClientMessage {
    fn parse_message(&self, _packet_id: u16) -> ClientMessage {
        self.clone()
    }
}
impl Connect {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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
    ) -> Connect {
        Connect {
            clean_start,
            last_will_flag,
            last_will_qos,
            last_will_retain,
            keep_alive,
            properties,
            client_id,
            will_properties: Some(will_properties),
            last_will_topic: Some(last_will_topic),
            last_will_message: Some(last_will_message),
            username: Some(username),
            password: Some(password.into_bytes()),
        }
    }

    /// Devuelve, en caso de que haya, un last will. Si el will flag está seteado en false, devuelve None
    pub fn give_will_message(self) -> Option<LastWill> {
        if !self.last_will_flag {
            return None;
        }

        let last_will_topic = self.last_will_topic?;
        let last_will_message = self.last_will_message?;
        let will_properties = self.will_properties?;

        Some(LastWill::new(
            last_will_topic,
            last_will_message,
            self.last_will_qos,
            self.last_will_retain,
            will_properties,
        ))
    }
    /// Abre un archivo de configuracion con propiedades y guarda sus lecturas.
    pub fn read_connect_config(file_path: &str) -> Result<Connect, ProtocolError> {
        let config_file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => {
                return Err(ProtocolError::ReadingConfigFileError);
            }
        };

        let reader: BufReader<File> = BufReader::new(config_file);
        let connect: Connect = match serde_json::from_reader(reader) {
            Ok(c) => c,
            Err(_) => return Err(ProtocolError::ReadingConfigFileError),
        };

        connect.check_will_properties()?;

        Ok(connect)
    }

    /// Abre un archivo de configuracion con propiedades y guarda sus lecturas.
    pub fn read_connect(file_path: &str) -> Result<Connect, ProtocolError> {
        let config_file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => return Err(ProtocolError::ReadingConfigFileError),
        };

        let reader: BufReader<File> = BufReader::new(config_file);
        let config: Connect = match serde_json::from_reader(reader) {
            Ok(c) => c,
            Err(_) => return Err(ProtocolError::ReadingConfigFileError),
        };

        config.check_will_properties()?;

        Ok(config)
    }

    fn check_will_properties(&self) -> Result<(), ProtocolError> {
        if let (Some(_), Some(_), Some(_)) = (
            &self.will_properties,
            &self.last_will_topic,
            &self.last_will_message,
        ) {
            if self.last_will_qos > 1 {
                //si es una QoS no soportada...
                return Err(ProtocolError::InvalidQOS);
            }
            return Ok(());
        }

        Err(ProtocolError::MissingWillMessageProperties)
    }

    pub fn write_to(&self, writer: &mut dyn Write) -> Result<(), ProtocolError> {
        //fixed header
        let byte_1: u8 = 0x10_u8.to_le(); //00010000
        let _ = writer
            .write_all(&[byte_1])
            .map_err(|_e| ProtocolError::WriteError);

        //protocol name
        let protocol_name = "MQTT";
        write_string(writer, protocol_name)?;

        //protocol version
        let protocol_version: u8 = 0x05;
        let _ = writer
            .write_all(&[protocol_version])
            .map_err(|_e| ProtocolError::WriteError);
        //connection flags
        let mut connect_flags: u8 = 0x00;
        if self.clean_start {
            connect_flags |= 1 << 1; //set bit 1 to 1
        }

        if self.last_will_flag {
            connect_flags |= 1 << 2;
        }
        if self.last_will_qos > 1 {
            return Err(ProtocolError::InvalidQOS);
        }
        connect_flags |= (self.last_will_qos & 0b11) << 3;

        if self.last_will_retain {
            connect_flags |= 1 << 5;
        }

        if let Some(password) = self.password.as_ref() {
            if !password.is_empty() {
                connect_flags |= 1 << 6;
            }
        }

        if let Some(username) = self.username.as_ref() {
            if !username.is_empty() {
                connect_flags |= 1 << 7;
            }
        }

        let _ = writer
            .write_all(&[connect_flags])
            .map_err(|_e| ProtocolError::WriteError);

        //keep alive
        write_u16(writer, &self.keep_alive)?;
        write_string(writer, &self.client_id)?;

        if let (Some(will_properties), Some(last_will_topic), Some(last_will_message)) = (
            self.will_properties.clone(),
            self.last_will_topic.clone(),
            self.last_will_message.clone(),
        ) {
            if self.last_will_flag {
                will_properties.write_to(writer)?;
                write_string(writer, &last_will_topic)?;
                write_string(writer, &last_will_message)?;
            }
        }

        if let Some(username) = self.username.as_ref() {
            if !username.is_empty() {
                write_string(writer, username)?;
            }
        }

        if let Some(password) = self.password.as_ref() {
            if !password.is_empty() {
                write_bin_vec(writer, password)?;
            }
        }

        self.properties.write_to(writer)?;

        Ok(())
    }

    pub fn read_from(stream: &mut impl Read) -> Result<Connect, Error> {
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
        //payload
        //client ID
        let client_id = read_string(stream)?;

        let mut last_will_topic = String::new();
        let mut will_message = String::new();

        let will_properties = WillProperties::read_from(stream)?;
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
        let mut pass = Vec::new();
        if hay_pass {
            pass = read_bin_vec(stream)?;
        }

        //properties
        let properties: ConnectProperties = ConnectProperties::read_from(stream)?;

        Ok(Connect {
            clean_start,
            last_will_flag,
            last_will_qos,
            last_will_retain,
            keep_alive,
            properties,
            client_id,
            will_properties: Some(will_properties),
            last_will_topic: Some(last_will_topic),
            last_will_message: Some(will_message),
            username: Some(user),
            password: Some(pass),
        })
    }
}

impl ClientMessage {
    pub fn write_to(&self, stream: &mut dyn Write) -> Result<(), ProtocolError> {
        let mut writer = BufWriter::new(stream);
        match self {
            ClientMessage::Connect(connect) => {
                connect.write_to(&mut writer)?;
                let _ = writer.flush().map_err(|_e| ProtocolError::WriteError);

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

                if *retain_flag == 1 {
                    //we must replace any existing retained message for this topic and store
                    //the app message.
                    byte_1 |= 1 << 0;
                }

                if *qos == 1 {
                    byte_1 |= 1 << 1;
                    byte_1 |= 0 << 2;
                } else if *qos != 0x00 && *qos != 0x01 {
                    return Err(ProtocolError::InvalidQOS);
                }

                if *dup_flag == 1 {
                    byte_1 |= 1 << 3;
                }

                //Dup flag must be set to 0 for all QoS 0 messages.
                if *qos == 0x00 {
                    byte_1 |= 0 << 3;
                }

                let _ = writer
                    .write_all(&[byte_1])
                    .map_err(|_e| ProtocolError::WriteError);

                //Remaining Length
                write_u16(&mut writer, packet_id)?;

                write_string(&mut writer, topic_name)?;

                //Properties
                let _ = properties
                    .write_properties(&mut writer)
                    .map_err(|_e| ProtocolError::WriteError);

                //Payload
                payload.write_to(&mut writer)?;

                let _ = writer.flush().map_err(|_e| ProtocolError::WriteError);
                Ok(())
            }
            ClientMessage::Subscribe {
                packet_id,
                properties,
                payload,
            } => {
                // fixed header
                let byte_1: u8 = 0x82_u8;
                writer
                    .write_all(&[byte_1])
                    .map_err(|_e| ProtocolError::WriteError)?;

                // variable header
                write_u16(&mut writer, packet_id)?;

                //Properties
                properties.write_properties(&mut writer)?;

                // payload
                write_string(&mut writer, &payload.topic)?;
                write_string(&mut writer, &payload.client_id)?;

                writer.flush().map_err(|_e| ProtocolError::WriteError)?;
                Ok(())
            }
            ClientMessage::Unsubscribe {
                packet_id,
                properties,
                payload,
            } => {
                // fixed header
                self.write_first_packet_byte(&mut writer)?;

                // variable header
                write_u16(&mut writer, packet_id)?;

                // variable header
                properties.write_properties(&mut writer)?;

                // escribir payload
                write_string(&mut writer, &payload.topic)?;
                write_string(&mut writer, &payload.client_id)?;

                writer.flush().map_err(|_e| ProtocolError::WriteError)?;
                Ok(())
            }
            ClientMessage::Disconnect {
                reason_code,
                session_expiry_interval,
                reason_string,
                client_id,
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

                write_string(&mut writer, client_id)?;

                let _ = writer.flush().map_err(|_e| ProtocolError::WriteError);
                Ok(())
            }
            ClientMessage::Pingreq => {
                let byte_1: u8 = 0xC0_u8;
                let _ = writer
                    .write_all(&[byte_1])
                    .map_err(|_e| ProtocolError::WriteError);
                let _ = writer.flush().map_err(|_e| ProtocolError::WriteError);
                Ok(())
            }
            ClientMessage::Auth {
                reason_code,
                authentication_method: _,
                authentication_data: _,
                reason_string: _,
                user_properties: _,
            } => {
                println!("authenticating user...");
                self.write_first_packet_byte(&mut writer)?;

                write_u8(&mut writer, reason_code)?;
                self.write_packet_properties(&mut writer)?;
                writer.flush().map_err(|_e| ProtocolError::WriteError)?;

                Ok(())
            }
        }
    }

    fn write_first_packet_byte(
        &self,
        writer: &mut BufWriter<&mut dyn Write>,
    ) -> Result<(), ProtocolError> {
        match self {
            ClientMessage::Connect { .. } => {
                let byte_1: u8 = 0x10_u8.to_le(); //00010000
                writer
                    .write_all(&[byte_1])
                    .map_err(|_e| ProtocolError::WriteError)?;
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

                match writer.write_all(&[byte_1]) {
                    Ok(_) => {}
                    Err(_) => {
                        return Err(ProtocolError::WriteError);
                    }
                }
            }
            ClientMessage::Subscribe {
                packet_id: _,
                properties: _,
                payload: _,
            } => {
                let byte_1: u8 = 0x82_u8;
                println!("estoy mandnado el header {:?}", byte_1);
                writer
                    .write_all(&[byte_1])
                    .map_err(|_e| ProtocolError::WriteError)?;
            }
            ClientMessage::Unsubscribe {
                packet_id: _,
                properties: _,
                payload: _,
            } => {
                let byte_1: u8 = 0xA2_u8;
                writer
                    .write_all(&[byte_1])
                    .map_err(|_e| ProtocolError::WriteError)?;
            }
            ClientMessage::Disconnect {
                reason_code: _,
                session_expiry_interval: _,
                reason_string: _,
                client_id: _,
            } => {
                let byte_1: u8 = 0xE0_u8;
                writer
                    .write_all(&[byte_1])
                    .map_err(|_e| ProtocolError::WriteError)?;
            }
            ClientMessage::Pingreq => {
                let byte_1: u8 = 0xC0_u8;
                writer
                    .write_all(&[byte_1])
                    .map_err(|_e| ProtocolError::WriteError)?;
            }
            ClientMessage::Auth {
                reason_code: _,
                authentication_method: _,
                authentication_data: _,
                reason_string: _,
                user_properties: _,
            } => {
                let byte_1 = 0xF0_u8;
                writer
                    .write_all(&[byte_1])
                    .map_err(|_e| ProtocolError::WriteError)?;
            }
        }
        Ok(())
    }

    fn write_packet_properties(
        &self,
        mut writer: &mut BufWriter<&mut dyn Write>,
    ) -> Result<(), ProtocolError> {
        match self {
            ClientMessage::Connect { .. } => Ok(()),
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
                Ok(())
            }
            ClientMessage::Subscribe {
                packet_id,
                properties,
                payload,
            } => {
                write_u16(writer, packet_id)?;

                //Properties
                properties.write_properties(writer)?;

                //payload
                write_string(writer, &payload.topic)?;
                write_string(writer, &payload.client_id)?;

                Ok(())
            }
            ClientMessage::Unsubscribe {
                packet_id,
                properties,
                payload,
            } => {
                write_u16(writer, packet_id)?;

                // variable header
                properties.write_properties(writer)?;

                //payload
                write_string(&mut writer, &payload.topic)?;
                write_string(&mut writer, &payload.client_id)?;

                Ok(())
            }
            ClientMessage::Disconnect {
                reason_code,
                session_expiry_interval,
                reason_string,
                client_id,
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

                write_string(writer, client_id)?;

                writer.flush().map_err(|_e| ProtocolError::WriteError)?;

                Ok(())
            }
            ClientMessage::Pingreq => {
                let byte_1: u8 = 0xC0_u8;
                writer
                    .write_all(&[byte_1])
                    .map_err(|_e| ProtocolError::WriteError)?;
                writer.flush().map_err(|_e| ProtocolError::WriteError)?;
                Ok(())
            }
            ClientMessage::Auth {
                reason_code,
                authentication_method,
                authentication_data,
                reason_string,
                user_properties,
            } => {
                let byte_1 = 0xF0_u8;
                let _ = writer
                    .write_all(&[byte_1])
                    .map_err(|_e| ProtocolError::WriteError);

                write_u8(&mut writer, reason_code).map_err(|_e| ProtocolError::WriteError)?;

                let authentication_method_id: u8 = 0x15_u8;
                let _ = writer
                    .write_all(&[authentication_method_id])
                    .map_err(|_e| ProtocolError::WriteError);
                write_string(&mut writer, authentication_method)?;

                let authentication_data_id: u8 = 0x16_u8;
                let _ = writer
                    .write_all(&[authentication_data_id])
                    .map_err(|_e| ProtocolError::WriteError);
                write_bin_vec(&mut writer, authentication_data)?;

                let reason_string_id: u8 = 0x1F_u8;
                let _ = writer
                    .write_all(&[reason_string_id])
                    .map_err(|_e| ProtocolError::WriteError);
                write_string(&mut writer, reason_string)?;

                let user_properties_id: u8 = 0x26_u8; // 38
                let _ = writer
                    .write_all(&[user_properties_id])
                    .map_err(|_e| ProtocolError::WriteError);
                write_tuple_vec(&mut writer, user_properties)?;

                let _ = writer.flush().map_err(|_e| ProtocolError::WriteError);

                Ok(())
            }
        }
    }

    pub fn read_from(stream: &mut impl Read) -> Result<ClientMessage, Error> {
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

                let connect = Connect::read_from(stream)?;

                Ok(ClientMessage::Connect(connect))
            }
            0x30 => {
                println!("Reading publish message");

                let packet_id = read_u16(stream)?;
                let topic_name = read_string(stream)?;
                let properties = PublishProperties::read_from(stream)?;

                let payload = PayloadTypes::read_from(stream)?;

                Ok(ClientMessage::Publish {
                    packet_id,
                    topic_name,
                    qos,
                    retain_flag,
                    payload,
                    dup_flag,
                    properties,
                })
            }
            0x82 => {
                let packet_id = read_u16(stream)?;

                let properties = SubscribeProperties::read_properties(stream)?;

                let payload = Subscription {
                    topic: read_string(stream)?,
                    client_id: read_string(stream)?,
                };

                Ok(ClientMessage::Subscribe {
                    packet_id,
                    properties,
                    payload,
                })
            }
            0xA2 => {
                let packet_id = read_u16(stream)?;

                let properties = SubscribeProperties::read_properties(stream)?;

                let payload = Subscription {
                    topic: read_string(stream)?,
                    client_id: read_string(stream)?,
                };

                Ok(ClientMessage::Unsubscribe {
                    packet_id,
                    properties,
                    payload,
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

                let client_id = read_string(stream)?;

                Ok(ClientMessage::Disconnect {
                    reason_code,
                    session_expiry_interval,
                    reason_string,
                    client_id,
                })
            }
            0xC0 => Ok(ClientMessage::Pingreq),
            0xF0 => {
                let reason_code = read_u8(stream)?;
                let mut authentication_method: Option<String> = None;
                let mut authentication_data: Option<Vec<u8>> = None;
                let mut reason_string: Option<String> = None;
                let mut user_properties: Option<Vec<(String, String)>> = None;

                let mut count = 0;
                while let Ok(property_id) = read_u8(stream) {
                    match property_id {
                        0x15 => {
                            let value = read_string(stream)?;
                            authentication_method = Some(value);
                        }
                        0x16 => {
                            let value = read_bin_vec(stream)?;
                            authentication_data = Some(value);
                        }
                        0x26 => {
                            let value = read_tuple_vec(stream)?;
                            user_properties = Some(value);
                        }
                        0x1F => {
                            let value = read_string(stream)?;
                            reason_string = Some(value);
                        }
                        _ => {
                            return Err(Error::new(ErrorKind::InvalidData, "Property ID inválido"));
                        }
                    }
                    count += 1;
                    if count == 4 {
                        break;
                    }
                }

                Ok(ClientMessage::Auth {
                    reason_code,
                    user_properties: user_properties.ok_or(Error::new(
                        ErrorKind::InvalidData,
                        "Missing user_properties property",
                    ))?,
                    authentication_method: authentication_method.ok_or(Error::new(
                        ErrorKind::InvalidData,
                        "Missing authentication_method property",
                    ))?,
                    authentication_data: authentication_data.ok_or(Error::new(
                        ErrorKind::InvalidData,
                        "Missing authentication_data property",
                    ))?,
                    reason_string: reason_string.ok_or(Error::new(
                        ErrorKind::InvalidData,
                        "Missing reason_string property",
                    ))?,
                })
            }
            _ => Err(Error::new(std::io::ErrorKind::Other, "Invalid header")),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{
        monitoring::incident::Incident,
        mqtt::{
            connect::{connect_properties::ConnectProperties, will_properties::WillProperties},
            publish::publish_properties::TopicProperties,
        },
        utils::{incident_payload::IncidentPayload, location::Location},
    };

    use super::*;
    fn read_json_to_connect_config(json_data: &str) -> Result<Connect, Box<dyn std::error::Error>> {
        let connect_config: Connect = serde_json::from_str(json_data)?;
        Ok(connect_config)
    }
    #[test]
    fn test_01_connect_message_ok() {
        let connect_read = Connect::read_connect("./src/monitoring/connect_config.json").unwrap();

        let connect = ClientMessage::Connect(connect_read.clone());

        let mut cursor = Cursor::new(Vec::<u8>::new());
        match connect.write_to(&mut cursor) {
            Ok(_) => {}
            Err(e) => {
                panic!("no se pudo escribir en el cursor {:?}", e);
            }
        }

        cursor.set_position(0);

        match ClientMessage::read_from(&mut cursor) {
            Ok(_) => {
                assert_eq!(
                    connect,
                    crate::mqtt::client_message::ClientMessage::Connect(connect_read)
                );
            }
            Err(e) => {
                panic!("no se pudo leer del cursor {:?}", e);
            }
        }
    }

    #[test]
    fn test_02_connect_without_props_err() {
        let connect = ClientMessage::Connect(Connect::new(
            true,
            true,
            1,
            true,
            35,
            ConnectProperties::new(
                30,
                1,
                20,
                20,
                true,
                true,
                vec![("hola".to_string(), "chau".to_string())],
                "password-based".to_string(),
                vec![1, 2, 3],
            ),
            "kvtr33".to_string(),
            WillProperties::new(
                1,
                1,
                1,
                "a".to_string(),
                "a".to_string(),
                [1, 2, 3].to_vec(),
                vec![("a".to_string(), "a".to_string())],
            ),
            "camera system".to_string(),
            "soy el monitoring y me desconecte".to_string(),
            "a".to_string(),
            "a".to_string(),
        ));
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

        let location = Location::new(12.1, 25.0);
        let incident = Incident::new(location);

        let payload = PayloadTypes::IncidentLocation(IncidentPayload::new(incident));

        let publish = ClientMessage::Publish {
            packet_id: 1,
            topic_name: "mensajes para juan".to_string(),
            qos: 1,
            retain_flag: 1,
            payload,
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
                // println!("{:?}", read_publish);
                assert_eq!(publish, read_publish);
            }
            Err(e) => {
                panic!("no se pudo leer del cursor {:?}", e);
            }
        }
    }

    #[test]
    fn test_04_subscribe_ok() {
        let payload = Subscription {
            topic: "topic".to_string(),
            client_id: "client".to_string(),
        };

        let sub = ClientMessage::Subscribe {
            packet_id: 1,
            properties: SubscribeProperties::new(
                1,
                vec![("propiedad".to_string(), "valor".to_string())],
            ),
            payload,
        };

        let mut cursor = Cursor::new(Vec::<u8>::new());
        match sub.write_to(&mut cursor) {
            Ok(_) => {}
            Err(e) => {
                panic!("no se pudo escribir en el cursor {:?}", e);
            }
        };
        cursor.set_position(0);
        let read_sub = match ClientMessage::read_from(&mut cursor) {
            Ok(sub) => sub,
            Err(e) => {
                panic!("no se pudo leer del cursor {:?}", e);
            }
        };

        println!("{:?}", read_sub);
        println!("{:?}", sub);
        assert_eq!(sub, read_sub);
    }

    #[test]
    fn test_05_unsubscribe_ok() {
        let payload = Subscription {
            topic: "topic".to_string(),
            client_id: "client".to_string(),
        };

        let unsub = ClientMessage::Unsubscribe {
            packet_id: 1,
            properties: SubscribeProperties::new(
                1,
                vec![("propiedad".to_string(), "valor".to_string())],
            ),
            payload,
        };

        let mut cursor = Cursor::new(Vec::<u8>::new());
        match unsub.write_to(&mut cursor) {
            Ok(_) => {}
            Err(e) => {
                panic!("no se pudo escribir en el cursor {:?}", e);
            }
        }
        cursor.set_position(0);
        let read_unsub = match ClientMessage::read_from(&mut cursor) {
            Ok(unsub) => unsub,
            Err(e) => {
                panic!("no se pudo leer del cursor {:?}", e);
            }
        };

        assert_eq!(unsub, read_unsub);
    }

    #[test]
    fn test_06_disconnect_ok() {
        let disconect = ClientMessage::Disconnect {
            reason_code: 1,
            session_expiry_interval: 1,
            reason_string: "hola".to_string(),
            client_id: "client".to_string(),
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

    // #
    // fn test_07_auth_ok() {
    //     let auth = ClientMessage::Auth {
    //         reason_code: 0x00_u8,
    //         authentication_method: "password-based".to_string(),
    //         authentication_data: vec![],
    //         reason_string: "usuario no encontrado".to_string(),
    //         user_properties: vec![("propiedad".to_string(), "valor".to_string())],
    //     };

    //     let mut cursor = Cursor::new(Vec::<u8>::new());
    //     match auth.write_to(&mut cursor) {
    //         Ok(_) => {}
    //         Err(e) => {
    //             panic!("no se pudo escribir en el cursor {:?}", e);
    //         }
    //     }
    //     cursor.set_position(0);
    //     let read_auth = match ClientMessage::read_from(&mut cursor) {
    //         Ok(auth) => auth,
    //         Err(e) => {
    //             panic!("no se pudo leer del cursor {:?}", e);
    //         }
    //     };
    //     assert_eq!(auth, read_auth);
    // }

    #[test]
    fn test_07_connect_with_invalid_qos_throws_err() -> std::io::Result<()> {
        let connect = ClientMessage::Connect(Connect::new(
            true,
            true,
            1,
            true,
            35,
            ConnectProperties::new(
                30,
                1,
                20,
                20,
                true,
                true,
                vec![("hola".to_string(), "chau".to_string())],
                "password-based".to_string(),
                vec![1, 2, 3],
            ),
            "kvtr33".to_string(),
            WillProperties::new(
                1,
                1,
                1,
                "a".to_string(),
                "a".to_string(),
                [1, 2, 3].to_vec(),
                vec![("a".to_string(), "a".to_string())],
            ),
            "camera system".to_string(),
            "soy el monitoring y me desconecte".to_string(),
            "a".to_string(),
            "a".to_string(),
        ));
        let mut cursor = Cursor::new(Vec::<u8>::new());

        match connect.write_to(&mut cursor) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("QoS invalida");
                assert_eq!(e, ProtocolError::InvalidQOS);
                Ok(())
            }
        }
    }

    #[test]
    fn test_01_config_creation_cases() {
        let config_ok = Connect::read_connect_config("./src/monitoring/connect_config.json");

        let config_err = Connect::read_connect_config("este/es/un/path/feo");

        assert!(config_ok.is_ok());
        assert!(config_err.is_err());
    }

    #[test]
    fn test_02_config_without_last_will_msg_throws_err() {
        let config_err = Connect::read_connect_config(
            "./tests/connect_config_test/config_without_will_msg.json",
        );

        assert!(config_err.is_err());
    }

    #[test]
    fn test_03_config_with_lat_will_invalid_qos_err() {
        let config_err = Connect::read_connect_config(
            "./tests/connect_config_test/connect_config_invalid_qos.json",
        );

        assert!(config_err.is_err());
    }

    #[test]
    fn test_read_json_connect_config() {
        let json_data = r#"{
            "clean_start": true,
            "last_will_flag": true,
            "last_will_qos": 1,
            "last_will_retain": true,
            "keep_alive": 35,
            "properties": {
                "session_expiry_interval": 30,
                "receive_maximum": 1,
                "maximum_packet_size": 20,
                "topic_alias_maximum": 20,
                "request_response_information": true,
                "request_problem_information": true,
                "user_properties": [
                    [
                        "hola",
                        "chau"
                    ]
                ],
                "authentication_method": "password-based",
                "authentication_data": [
                    1,
                    2,
                    3
                ]
            },
            "client_id": "kvtr33",
            "will_properties": {
                "last_will_delay_interval": 1,
                "payload_format_indicator": 1,
                "message_expiry_interval": 1,
                "content_type": "a",
                "response_topic": "a",
                "correlation_data": [
                    1,
                    2,
                    3
                ],
                "user_properties": [
                    [
                        "a",
                        "a"
                    ]
                ]
            },
            "last_will_topic": "camera system",
            "last_will_message": "soy el monitoring y me desconecte",
            "username": "a",
            "password": [97]
        }"#;
        let connect_config = read_json_to_connect_config(json_data).unwrap();
        let expected_connect_config = Connect::new(
            true,
            true,
            1,
            true,
            35,
            ConnectProperties::new(
                30,
                1,
                20,
                20,
                true,
                true,
                vec![("hola".to_string(), "chau".to_string())],
                "password-based".to_string(),
                vec![1, 2, 3],
            ),
            "kvtr33".to_string(),
            WillProperties::new(
                1,
                1,
                1,
                "a".to_string(),
                "a".to_string(),
                [1, 2, 3].to_vec(),
                vec![("a".to_string(), "a".to_string())],
            ),
            "camera system".to_string(),
            "soy el monitoring y me desconecte".to_string(),
            "a".to_string(),
            "a".to_string(),
        );
        assert_eq!(connect_config, expected_connect_config);
    }

    #[test]
    fn test_parse_message() {
        let connect_properties = ConnectProperties::new(
            30,
            1,
            20,
            20,
            true,
            true,
            vec![("hola".to_string(), "chau".to_string())],
            "auth".to_string(),
            vec![1, 2, 3],
        );

        let will_properties = WillProperties::new(
            1,
            1,
            1,
            "a".to_string(),
            "a".to_string(),
            [1, 2, 3].to_vec(),
            vec![("a".to_string(), "a".to_string())],
        );

        let connect_config = Connect::new(
            true,
            true,
            1,
            true,
            35,
            connect_properties.clone(),
            "juancito".to_string(),
            will_properties.clone(),
            "camera system".to_string(),
            "soy el monitoring y me desconecte".to_string(),
            "a".to_string(),
            "a".to_string(),
        );

        let connect_message = connect_config.parse_message(1);

        assert_eq!(
            connect_message,
            ClientMessage::Connect(connect_config.clone())
        );
    }

    #[test]
    fn test_parse_pingreq() {
        let pingreq = ClientMessage::Pingreq;
        let pingreq_message = pingreq.parse_message(1);

        assert_eq!(pingreq_message, ClientMessage::Pingreq);
    }
}
