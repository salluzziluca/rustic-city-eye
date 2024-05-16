use std::io::{Error, Read, Write};

use crate::mqtt::writer::*;
use crate::mqtt::reader::*;

#[derive(Debug, PartialEq, Clone)]
pub struct SubscribeProperties{
    subscription_identifier: u32,
    user_properties: Vec<(String, String)>,
    payload: Vec<u8>,
}

impl SubscribeProperties {
    pub fn new(subscription_identifier: u32, user_properties: Vec<(String, String)>, payload: Vec<u8>) -> SubscribeProperties {
        SubscribeProperties {
            subscription_identifier,
            user_properties,
            payload,
        }
    }

   pub fn write_properties(&self, stream: &mut dyn Write) -> Result<(), Error> {
        write_u32(stream, &self.subscription_identifier)?;
        write_string_pairs(stream, &self.user_properties)?;
        write_bin_vec(stream, &self.payload)?;
        Ok(())
    }

    pub fn read_properties(stream: &mut dyn Read) -> Result<SubscribeProperties, Error> {
        let subscription_identifier = read_u32(stream)?;
        let user_properties = read_string_pairs(stream)?;
        let payload = read_bin_vec(stream)?;
        Ok(SubscribeProperties::new(subscription_identifier, user_properties, payload))
    }
}