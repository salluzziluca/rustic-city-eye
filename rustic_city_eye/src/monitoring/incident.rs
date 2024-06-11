use crate::{mqtt::protocol_error::ProtocolError, utils::{location::Location, writer::write_string}};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Incident {
    location: Location,
}

impl Incident {
    pub fn new(location: Location) -> Incident {
        Self { location }
    }

    pub fn get_location(&self) -> Location {
        self.location.clone()
    }

    pub fn write_to(&self, stream: &mut dyn std::io::prelude::Write) -> Result<(), ProtocolError> {
        let longitude_string = self.location.long.to_string();
        write_string(stream, &longitude_string)?;

        let latitude_string = self.location.lat.to_string();
        write_string(stream, &latitude_string)?;

        Ok(())
    }
}

#[cfg(test)]

mod tests {
    //use std::io::Cursor;

    use super::*;
    #[test]
    fn test_new_incident() {
        let location = Location::new(1.0, 2.0);
        let incident = Incident::new(location.clone());
        assert_eq!(incident.get_location(), location);
    }
}
