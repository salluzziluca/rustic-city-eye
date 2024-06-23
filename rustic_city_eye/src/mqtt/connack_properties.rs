use crate::utils::reader::*;
use crate::utils::writer::*;
use std::io::{BufWriter, Error, ErrorKind, Read, Write};

use super::protocol_error::ProtocolError;

#[derive(Debug, PartialEq)]
pub struct ConnackProperties {
    pub session_expiry_interval: u32,
    pub receive_maximum: u16,
    pub maximum_qos: bool,
    pub retain_available: bool,
    pub maximum_packet_size: u32,
    pub assigned_client_identifier: String,
    pub topic_alias_maximum: u16,
    pub reason_string: String,
    pub user_properties: Vec<(String, String)>,
    pub wildcard_subscription_available: bool,
    pub subscription_identifier_available: bool,
    pub shared_subscription_available: bool,
    pub server_keep_alive: u16,
    pub response_information: String,
    pub server_reference: String,
    pub authentication_method: String,
    pub authentication_data: Vec<u8>,
}

impl ConnackProperties {
    pub fn write_to(&self, stream: &mut dyn Write) -> Result<(), ProtocolError> {
        let mut writer = BufWriter::new(stream);

        let session_expiry_interval_id: u8 = 0x11_u8; //17
        print!(
            "session_expiry_interval_id: {:?}",
            session_expiry_interval_id
        );
        let _ = writer
            .write_all(&[session_expiry_interval_id])
            .map_err(|_| ProtocolError::WriteError);
        write_u32(&mut writer, &self.session_expiry_interval)?;

        let assigned_client_identifier_id: u8 = 0x12_u8; //18
        print!(
            "assigned_client_identifier_id: {:?}",
            assigned_client_identifier_id
        );
        let _ = writer
            .write_all(&[assigned_client_identifier_id])
            .map_err(|_| ProtocolError::WriteError);
        write_string(&mut writer, &self.assigned_client_identifier)?;

        let authentication_method_id: u8 = 0x15_u8; //21
        print!("authentication_method_id: {:?}", authentication_method_id);
        let _ = writer
            .write_all(&[authentication_method_id])
            .map_err(|_| ProtocolError::WriteError);
        write_string(&mut writer, &self.authentication_method)?;

        let authentication_data_id: u8 = 0x16_u8; //22
        print!("authentication_data_id: {:?}", authentication_data_id);
        let _ = writer
            .write_all(&[authentication_data_id])
            .map_err(|_| ProtocolError::WriteError);
        write_bin_vec(&mut writer, &self.authentication_data)?;

        let response_information_id: u8 = 0x1A_u8; //26
        print!("response_information_id: {:?}", response_information_id);
        let _ = writer
            .write_all(&[response_information_id])
            .map_err(|_| ProtocolError::WriteError);
        write_string(&mut writer, &self.response_information)?;

        let server_reference_id: u8 = 0x1C_u8; //28
        print!("server_reference_id: {:?}", server_reference_id);
        let _ = writer
            .write_all(&[server_reference_id])
            .map_err(|_: Error| ProtocolError::WriteError);
        write_string(&mut writer, &self.server_reference)?;

        let reason_string_id: u8 = 0x1F_u8; //31
        print!("reason_string_id: {:?}", reason_string_id);
        let _ = writer
            .write_all(&[reason_string_id])
            .map_err(|_| ProtocolError::WriteError);
        write_string(&mut writer, &self.reason_string)?;

        let receive_maximum_id: u8 = 0x21_u8; //33
        print!("receive_maximum_id: {:?}", receive_maximum_id);
        let _ = writer
            .write_all(&[receive_maximum_id])
            .map_err(|_| ProtocolError::WriteError);
        write_u16(&mut writer, &self.receive_maximum)?;

        let topic_alias_maximum_id: u8 = 0x22_u8; //34
        print!("topic_alias_maximum_id: {:?}", topic_alias_maximum_id);
        let _ = writer
            .write_all(&[topic_alias_maximum_id])
            .map_err(|_| ProtocolError::WriteError);
        write_u16(&mut writer, &self.topic_alias_maximum)?;

        let maximum_qos_id: u8 = 0x24_u8; //36
        print!("maximum_qos_id: {:?}", maximum_qos_id);
        let _ = writer
            .write_all(&[maximum_qos_id])
            .map_err(|_| ProtocolError::WriteError);
        write_bool(&mut writer, &self.maximum_qos)?;

        let retain_available_id: u8 = 0x25_u8; //37
        print!("retain_available_id: {:?}", retain_available_id);
        let _ = writer
            .write_all(&[retain_available_id])
            .map_err(|_| ProtocolError::WriteError);
        write_bool(&mut writer, &self.retain_available)?;

        let user_properties_id: u8 = 0x26_u8; //38
        print!("user_properties_id: {:?}", user_properties_id);
        let _ = writer
            .write_all(&[user_properties_id])
            .map_err(|_| ProtocolError::WriteError);
        write_tuple_vec(&mut writer, &self.user_properties)?;

        let maximum_packet_size_id: u8 = 0x27_u8; //39
        print!("maximum_packet_size_id: {:?}", maximum_packet_size_id);
        let _ = writer
            .write_all(&[maximum_packet_size_id])
            .map_err(|_| ProtocolError::WriteError);
        write_u32(&mut writer, &self.maximum_packet_size)?;

        let wildcard_subscription_available_id: u8 = 0x28_u8; //40
        print!(
            "wildcard_subscription_available_id: {:?}",
            wildcard_subscription_available_id
        );
        let _ = writer
            .write_all(&[wildcard_subscription_available_id])
            .map_err(|_| ProtocolError::WriteError);
        write_bool(&mut writer, &self.wildcard_subscription_available)?;

        let subscription_identifier_available_id: u8 = 0x29_u8; //41
        print!(
            "subscription_identifier_available_id: {:?}",
            subscription_identifier_available_id
        );
        let _ = writer
            .write_all(&[subscription_identifier_available_id])
            .map_err(|_| ProtocolError::WriteError);
        write_bool(&mut writer, &self.subscription_identifier_available)?;

        let shared_subscription_available_id: u8 = 0x2A_u8; //42
        print!(
            "shared_subscription_available_id: {:?}",
            shared_subscription_available_id
        );
        let _ = writer
            .write_all(&[shared_subscription_available_id])
            .map_err(|_| ProtocolError::WriteError);
        write_bool(&mut writer, &self.shared_subscription_available)?;

        let server_keep_alive_id: u8 = 0x2D_u8; //45
        print!("server_keep_alive_id: {:?}", server_keep_alive_id);
        let _ = writer
            .write_all(&[server_keep_alive_id])
            .map_err(|_| ProtocolError::WriteError);
        write_u16(&mut writer, &self.server_keep_alive)?;

        Ok(())
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<ConnackProperties, Error> {
        let session_expiry_interval_id = read_u8(stream)?;
        if session_expiry_interval_id != 0x11 {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid session expiry interval id",
            ));
        }
        let session_expiry_interval = read_u32(stream)?;

        let assigned_client_identifier_id = read_u8(stream)?;
        if assigned_client_identifier_id != 0x12 {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid assigned client identifier id",
            ));
        }
        let assigned_client_identifier = read_string(stream)?;

        let authentication_method_id = read_u8(stream)?;
        if authentication_method_id != 0x15 {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid authentication method id",
            ));
        }
        let authentication_method = read_string(stream)?;

        let authentication_data_id = read_u8(stream)?;
        if authentication_data_id != 0x16 {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid authentication data id",
            ));
        }
        let authentication_data = read_bin_vec(stream)?;

        let response_information_id = read_u8(stream)?;
        if response_information_id != 0x1A {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid response information id",
            ));
        }
        let response_information = read_string(stream)?;

        let server_reference_id = read_u8(stream)?;
        if server_reference_id != 0x1C {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid server reference id",
            ));
        }
        let server_reference = read_string(stream)?;

        let reason_string_id = read_u8(stream)?;
        if reason_string_id != 0x1F {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid reason string id",
            ));
        }
        let reason_string = read_string(stream)?;

        let receive_maximum_id = read_u8(stream)?;
        if receive_maximum_id != 0x21 {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid receive maximum id",
            ));
        }
        let receive_maximum = read_u16(stream)?;

        let topic_alias_maximum_id = read_u8(stream)?;
        if topic_alias_maximum_id != 0x22 {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid topic alias maximum id",
            ));
        }
        let topic_alias_maximum = read_u16(stream)?;

        let maximum_qos_id = read_u8(stream)?;
        if maximum_qos_id != 0x24 {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid maximum qos id",
            ));
        }
        let maximum_qos = read_bool(stream)?;

        let retain_available_id = read_u8(stream)?;
        if retain_available_id != 0x25 {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid retain available id",
            ));
        }
        let retain_available = read_bool(stream)?;

        let user_properties_id = read_u8(stream)?;
        if user_properties_id != 0x26 {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid user properties id",
            ));
        }
        let user_properties = read_tuple_vec(stream)?;

        let maximum_packet_size_id = read_u8(stream)?;
        if maximum_packet_size_id != 0x27 {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid maximum packet size id",
            ));
        }
        let maximum_packet_size = read_u32(stream)?;

        let wildcard_subscription_available_id = read_u8(stream)?;
        if wildcard_subscription_available_id != 0x28 {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid wildcard subscription available id",
            ));
        }
        let wildcard_subscription_available = read_bool(stream)?;

        let subscription_identifier_available_id = read_u8(stream)?;
        if subscription_identifier_available_id != 0x29 {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid subscription identifier available id",
            ));
        }
        let subscription_identifier_available = read_bool(stream)?;

        let shared_subscription_available_id = read_u8(stream)?;
        if shared_subscription_available_id != 0x2A {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid shared subscription available id",
            ));
        }
        let shared_subscription_available = read_bool(stream)?;

        let server_keep_alive_id = read_u8(stream)?;
        if server_keep_alive_id != 0x2D {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid server keep alive id",
            ));
        }
        let server_keep_alive = read_u16(stream)?;

        Ok(ConnackProperties {
            session_expiry_interval,
            assigned_client_identifier,
            authentication_method,
            authentication_data,
            response_information,
            server_reference,
            reason_string,
            receive_maximum,
            topic_alias_maximum,
            maximum_qos,
            retain_available,
            user_properties,
            maximum_packet_size,
            wildcard_subscription_available,
            subscription_identifier_available,
            shared_subscription_available,
            server_keep_alive,
        })
    }
}

pub struct ConnackPropertiesBuilder {
    session_expiry_interval: Option<u32>,
    receive_maximum: Option<u16>,
    maximum_qos: Option<bool>,
    retain_available: Option<bool>,
    maximum_packet_size: Option<u32>,
    assigned_client_identifier: Option<String>,
    topic_alias_maximum: Option<u16>,
    reason_string: Option<String>,
    user_properties: Option<Vec<(String, String)>>,
    wildcard_subscription_available: Option<bool>,
    subscription_identifier_available: Option<bool>,
    shared_subscription_available: Option<bool>,
    server_keep_alive: Option<u16>,
    response_information: Option<String>,
    server_reference: Option<String>,
    authentication_method: Option<String>,
    authentication_data: Option<Vec<u8>>,
}

impl Default for ConnackPropertiesBuilder {
    fn default() -> Self {
        ConnackPropertiesBuilder::new()
    }
}

impl ConnackPropertiesBuilder {
    pub fn new() -> ConnackPropertiesBuilder {
        ConnackPropertiesBuilder {
            session_expiry_interval: None,
            receive_maximum: None,
            maximum_qos: None,
            retain_available: None,
            maximum_packet_size: None,
            assigned_client_identifier: None,
            topic_alias_maximum: None,
            reason_string: None,
            user_properties: None,
            wildcard_subscription_available: None,
            subscription_identifier_available: None,
            shared_subscription_available: None,
            server_keep_alive: None,
            response_information: None,
            server_reference: None,
            authentication_method: None,
            authentication_data: None,
        }
    }

    pub fn session_expiry_interval(mut self, value: u32) -> Self {
        self.session_expiry_interval = Some(value);
        self
    }

    pub fn receive_maximum(mut self, value: u16) -> Self {
        self.receive_maximum = Some(value);
        self
    }

    pub fn maximum_qos(mut self, value: bool) -> Self {
        self.maximum_qos = Some(value);
        self
    }

    pub fn retain_available(mut self, value: bool) -> Self {
        self.retain_available = Some(value);
        self
    }

    pub fn maximum_packet_size(mut self, value: u32) -> Self {
        self.maximum_packet_size = Some(value);
        self
    }

    pub fn assigned_client_identifier(mut self, value: String) -> Self {
        self.assigned_client_identifier = Some(value);
        self
    }

    pub fn topic_alias_maximum(mut self, value: u16) -> Self {
        self.topic_alias_maximum = Some(value);
        self
    }

    pub fn reason_string(mut self, value: String) -> Self {
        self.reason_string = Some(value);
        self
    }

    pub fn user_properties(mut self, value: Vec<(String, String)>) -> Self {
        self.user_properties = Some(value);
        self
    }

    pub fn wildcard_subscription_available(mut self, value: bool) -> Self {
        self.wildcard_subscription_available = Some(value);
        self
    }

    pub fn subscription_identifier_available(mut self, value: bool) -> Self {
        self.subscription_identifier_available = Some(value);
        self
    }

    pub fn shared_subscription_available(mut self, value: bool) -> Self {
        self.shared_subscription_available = Some(value);
        self
    }

    pub fn server_keep_alive(mut self, value: u16) -> Self {
        self.server_keep_alive = Some(value);
        self
    }

    pub fn response_information(mut self, value: String) -> Self {
        self.response_information = Some(value);
        self
    }

    pub fn server_reference(mut self, value: String) -> Self {
        self.server_reference = Some(value);
        self
    }

    pub fn authentication_method(mut self, value: String) -> Self {
        self.authentication_method = Some(value);
        self
    }

    pub fn authentication_data(mut self, value: Vec<u8>) -> Self {
        self.authentication_data = Some(value);
        self
    }

    pub fn build(self) -> Result<ConnackProperties, Error> {
        Ok(ConnackProperties {
            session_expiry_interval: self.session_expiry_interval.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing session_expiry_interval property",
            ))?,
            receive_maximum: self.receive_maximum.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing receive_maximum property",
            ))?,
            maximum_qos: self.maximum_qos.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing maximum_qos property",
            ))?,
            retain_available: self.retain_available.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing retain_available property",
            ))?,
            maximum_packet_size: self.maximum_packet_size.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing maximum_packet_size property",
            ))?,
            assigned_client_identifier: self.assigned_client_identifier.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing assigned_client_identifier property",
            ))?,
            topic_alias_maximum: self.topic_alias_maximum.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing topic_alias_maximum property",
            ))?,
            reason_string: self.reason_string.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing reason_string property",
            ))?,
            user_properties: self.user_properties.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing user_properties property",
            ))?,
            wildcard_subscription_available: self.wildcard_subscription_available.ok_or(
                Error::new(
                    ErrorKind::InvalidData,
                    "Missing wildcard_subscription_available property",
                ),
            )?,
            subscription_identifier_available: self.subscription_identifier_available.ok_or(
                Error::new(
                    ErrorKind::InvalidData,
                    "Missing subscription_identifier_available property",
                ),
            )?,
            shared_subscription_available: self.shared_subscription_available.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing shared_subscription_available property",
            ))?,
            server_keep_alive: self.server_keep_alive.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing server_keep_alive property",
            ))?,
            response_information: self.response_information.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing response_information property",
            ))?,
            server_reference: self.server_reference.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing server_reference property",
            ))?,
            authentication_method: self.authentication_method.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing authentication_method property",
            ))?,
            authentication_data: self.authentication_data.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing authentication_data property",
            ))?,
        })
    }
}

#[cfg(test)]
#[test]

fn test_read_write() {
    let mut buffer = vec![];
    let properties = ConnackPropertiesBuilder::new()
        .session_expiry_interval(0)
        .receive_maximum(0)
        .maximum_qos(false)
        .retain_available(false)
        .maximum_packet_size(0)
        .assigned_client_identifier("propiedad".to_string())
        .topic_alias_maximum(0)
        .reason_string("propiedad".to_string())
        .user_properties(vec![("propiedad".to_string(), "valor".to_string())])
        .wildcard_subscription_available(false)
        .subscription_identifier_available(false)
        .shared_subscription_available(false)
        .server_keep_alive(0)
        .response_information("propiedad".to_string())
        .server_reference("propiedad".to_string())
        .authentication_method("propiedad".to_string())
        .authentication_data(vec![0, 1, 2, 3])
        .build()
        .unwrap();
    properties.write_to(&mut buffer).unwrap();
    let mut buffer = buffer.as_slice();
    let read_properties = ConnackProperties::read_from(&mut buffer).unwrap();
    assert_eq!(properties, read_properties);
}

#[test]
fn test_builder() {
    let properties = ConnackPropertiesBuilder::new()
        .session_expiry_interval(0)
        .receive_maximum(0)
        .maximum_qos(false)
        .retain_available(false)
        .maximum_packet_size(0)
        .assigned_client_identifier("propiedad".to_string())
        .topic_alias_maximum(0)
        .reason_string("propiedad".to_string())
        .user_properties(vec![("propiedad".to_string(), "valor".to_string())])
        .wildcard_subscription_available(false)
        .subscription_identifier_available(false)
        .shared_subscription_available(false)
        .server_keep_alive(0)
        .response_information("propiedad".to_string())
        .server_reference("propiedad".to_string())
        .authentication_method("propiedad".to_string())
        .authentication_data(vec![0, 1, 2, 3])
        .build()
        .unwrap();
    assert_eq!(properties.session_expiry_interval, 0);
    assert_eq!(properties.receive_maximum, 0);
    assert!(!properties.maximum_qos);
    assert!(!properties.retain_available);
    assert_eq!(properties.maximum_packet_size, 0);
    assert_eq!(
        properties.assigned_client_identifier,
        "propiedad".to_string()
    );
    assert_eq!(properties.topic_alias_maximum, 0);
    assert_eq!(properties.reason_string, "propiedad".to_string());
    assert_eq!(
        properties.user_properties,
        vec![("propiedad".to_string(), "valor".to_string())]
    );
    assert!(!properties.wildcard_subscription_available);
    assert!(!properties.subscription_identifier_available);
    assert!(!properties.shared_subscription_available);
    assert_eq!(properties.server_keep_alive, 0);
    assert_eq!(properties.response_information, "propiedad".to_string());
    assert_eq!(properties.server_reference, "propiedad".to_string());
    assert_eq!(properties.authentication_method, "propiedad".to_string());
    assert_eq!(properties.authentication_data, vec![0, 1, 2, 3]);
}
