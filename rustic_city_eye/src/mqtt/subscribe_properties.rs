use serde::{Deserialize, Serialize};

use crate::utils::{reader::*, writer::*};
use std::io::{Error, Read, Write};

use super::protocol_error::ProtocolError;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]

pub struct SubscribeProperties {
    pub sub_id: u8,
    user_properties: Vec<(String, String)>,
}

impl SubscribeProperties {
    pub fn new(sub_id: u8, user_properties: Vec<(String, String)>) -> SubscribeProperties {
        SubscribeProperties {
            sub_id,
            user_properties,
        }
    }

    pub fn write_properties(&self, stream: &mut dyn Write) -> Result<(), ProtocolError> {
        write_u8(stream, &self.sub_id)?;
        write_string_pairs(stream, &self.user_properties)?;
        Ok(())
    }

    pub fn read_properties(stream: &mut dyn Read) -> Result<SubscribeProperties, Error> {
        let sub_id = read_u8(stream)?;
        let user_properties = read_string_pairs(stream)?;
        Ok(SubscribeProperties::new(sub_id, user_properties))
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_new_subscribe_properties() {
        let user_properties = vec![("key".to_string(), "value".to_string())];
        let subscribe_properties = SubscribeProperties::new(1, user_properties.clone());
        assert_eq!(subscribe_properties.sub_id, 1);
        assert_eq!(subscribe_properties.user_properties, user_properties);
    }

    #[test]
    fn test_read_properties() {
        let user_properties = vec![("key".to_string(), "value".to_string())];
        let subscribe_properties = SubscribeProperties::new(1, user_properties.clone());
        let mut cursor = Cursor::new(Vec::new());
        subscribe_properties.write_properties(&mut cursor).unwrap();
        cursor.set_position(0);
        let read_subscribe_properties = SubscribeProperties::read_properties(&mut cursor).unwrap();
        assert_eq!(read_subscribe_properties, subscribe_properties);
    }
}
