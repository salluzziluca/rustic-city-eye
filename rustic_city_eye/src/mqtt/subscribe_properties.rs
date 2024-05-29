use std::io::{Error, Read, Write};

use crate::mqtt::reader::*;
use crate::mqtt::writer::*;

#[derive(Debug, PartialEq, Clone)]
pub struct SubscribeProperties {
    pub sub_id: u8,
    user_properties: Vec<(String, String)>,
    payload: Vec<u8>,
}

impl SubscribeProperties {
    pub fn new(
        sub_id: u8,
        user_properties: Vec<(String, String)>,
        payload: Vec<u8>,
    ) -> SubscribeProperties {
        SubscribeProperties {
            sub_id,
            user_properties,
            payload,
        }
    }

    pub fn write_properties(&self, stream: &mut dyn Write) -> Result<(), Error> {
        write_u8(stream, &self.sub_id)?;
        write_string_pairs(stream, &self.user_properties)?;
        write_bin_vec(stream, &self.payload)?;
        Ok(())
    }

    pub fn read_properties(stream: &mut dyn Read) -> Result<SubscribeProperties, Error> {
        let sub_id = read_u8(stream)?;
        let user_properties = read_string_pairs(stream)?;
        let payload = read_bin_vec(stream)?;
        Ok(SubscribeProperties::new(sub_id, user_properties, payload))
    }
}
