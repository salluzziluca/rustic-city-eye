use crate::mqtt::writer::*;
use crate::mqtt::reader::*;

use std::io::ErrorKind;
use std::io::{BufReader, BufWriter, Error, Read, Write};

struct ConnectProperties {
    session_expiry_interval: u32,
    receive_maximum: u16,
    maximum_packet_size: u32,
    topic_alias_maximum: u16,
    request_response_information: bool,
    request_problem_information: bool,
    user_properties: Vec<(String, String)>,
    authentication_method: String,
    authentication_data: Vec<u8>,
}

impl ConnectProperties {
    pub fn write_to(&self, stream: &mut dyn Write) -> Result<(), Error> {
        let mut writer = BufWriter::new(stream);

        let session_expiry_interval_id: u8 = 0x11_u8;
        writer.write(&[session_expiry_interval_id])?;
        write_u32(&mut writer, &self.session_expiry_interval)?;

        let authentication_method_id: u8 = 0x15_u8;
        writer.write(&[authentication_method_id])?;
        write_string(&mut writer, &self.authentication_method)?;

        let authentication_data_id: u8 = 0x16_u8;
        writer.write(&[authentication_data_id])?;
        write_bin_vec(&mut writer, &self.authentication_data)?;

        let request_problem_information_id: u8 = 0x17_u8;
        writer.write(&[request_problem_information_id])?;
        write_bool(&mut writer, &self.request_problem_information)?;

        let request_response_information_id: u8 = 0x19_u8; // 25
        writer.write(&[request_response_information_id])?;
        write_bool(&mut writer, &self.request_response_information)?;

        let receive_maximum_id: u8 = 0x21_u8; // 33
        writer.write(&[receive_maximum_id])?;
        write_u16(&mut writer, &self.receive_maximum)?;

        let topic_alias_maximum_id: u8 = 0x22_u8; // 34
        writer.write(&[topic_alias_maximum_id])?;
        write_u16(&mut writer, &self.topic_alias_maximum)?;

        let user_properties_id: u8 = 0x26_u8; // 38
        writer.write(&[user_properties_id])?;
        write_tuple_vec(&mut writer, &self.user_properties)?;

        let maximum_packet_size_id: u8 = 0x27_u8; // 39
        writer.write(&[maximum_packet_size_id])?;
        write_u32(&mut writer, &self.maximum_packet_size)?;

        Ok(())
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<ConnectProperties, Error> {
        let mut reader = BufReader::new(stream);

        let mut session_expiry_interval: Option<u32> = None;
        let mut receive_maximum: Option<u16> = None;
        let mut maximum_packet_size: Option<u32> = None;
        let mut topic_alias_maximum: Option<u16> = None;
        let mut request_response_information: Option<bool> = None;
        let mut request_problem_information: Option<bool> = None;
        let mut user_properties: Option<Vec<(String, String)>> = None;
        let mut authentication_method: Option<String> = None;
        let mut authentication_data: Option<Vec<u8>> = None;

        while let Ok(property_id) = read_u8(&mut reader){
            match property_id {
                0x11 => {
                    let value = read_u32(&mut reader)?;
                    session_expiry_interval = Some(value);
                }
                0x15 => {
                    let value = read_string(&mut reader)?;
                    authentication_method = Some(value);
                }
                0x16 => {
                    let value = read_bin_vec(&mut reader)?;
                    authentication_data = Some(value);
                }
                0x17 => {
                    let value = read_bool(&mut reader)?;
                    request_problem_information = Some(value);
                }
                0x19 => {
                    let value = read_bool(&mut reader)?;
                    request_response_information = Some(value);
                }
                0x21 => {
                    let value = read_u16(&mut reader)?;
                    receive_maximum = Some(value);
                }
                0x22 => {
                    let value = read_u16(&mut reader)?;
                    topic_alias_maximum = Some(value);
                }
                0x26 => {
                    let value = read_tuple_vec(&mut reader)?;
                    user_properties = Some(value);
                }
                0x27 => {
                    let value = read_u32(&mut reader)?;
                    maximum_packet_size = Some(value);
                }
                _ => {
                    return Err(Error::new(ErrorKind::InvalidData, "Invalid property id"));
                }
            }
        }

        Ok(ConnectProperties {
            session_expiry_interval: session_expiry_interval.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing session_expiry_interval property",
            ))?,
            receive_maximum: receive_maximum.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing receive_maximum property",
            ))?,
            maximum_packet_size: maximum_packet_size.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing maximum_packet_size property",
            ))?,
            topic_alias_maximum: topic_alias_maximum.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing topic_alias_maximum property",
            ))?,
            request_response_information: request_response_information.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing request_response_information property",
            ))?,
            request_problem_information: request_problem_information.ok_or(Error::new(
                ErrorKind::InvalidData,
                "Missing request_problem_information property",
            ))?,
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
        })
    }
}
