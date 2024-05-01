use crate::mqtt::writer::*;
use std::io::{BufWriter, Error, Write};

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
        //write_vec_bin // ?????????????????????

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

    // pub fn read_from(stream: &mut dyn Read) -> Result<connectProperties, Error> {}
}
