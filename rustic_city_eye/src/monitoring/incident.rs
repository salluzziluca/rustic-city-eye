use crate::{utils::location::Location, utils::writer::write_string};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Incident {
    location: Location,
}

impl Incident {
    pub fn new(location: Location) -> Incident {
        Self { location }
    }

    // pub fn parse_to_string(&self) -> String {
    //     format!("Incident at {}", self.location.parse_to_string())
    // }

    // pub fn from_string(incident: String) -> Incident {
    //     let location = incident.split(" at ").collect::<Vec<&str>>()[1];
    //     Incident {
    //         location: Location::new(
    //             location.split(", ").collect::<Vec<&str>>()[0].to_string(),
    //             location.split(", ").collect::<Vec<&str>>()[1].to_string(),
    //         ),
    //     }
    // }

    pub fn get_location(&self) -> Location {
        self.location.clone()
    }

    pub fn write_to(&self, stream: &mut dyn std::io::prelude::Write) -> std::io::Result<()> {
        let longitude_string = self.location.longitude.to_string();
        write_string(stream, &longitude_string)?;

        let latitude_string = self.location.latitude.to_string();
        write_string(stream, &latitude_string)?;

        Ok(())
    }
}

#[cfg(test)]

mod tests {
    use std::io::Cursor;

    use super::*;
    #[test]
    fn test_new_incident() {
        let location = Location::new("1.0".to_string(), "2.0".to_string());
        let incident = Incident::new(location.clone());
        assert_eq!(incident.get_location(), location);
    }

    #[test]
    fn test_parse_to_string() {
        let location = Location::new("1.0".to_string(), "2.0".to_string());
        let incident = Incident::new(location.clone());

        assert_eq!(incident.parse_to_string(), "Incident at (1, 2)");
    }

    #[test]
    fn test_from_string() {
        let location = Location::new("1.0".to_string(), "2.0".to_string());
        let incident = Incident::new(location.clone());
        assert_eq!(
            Incident::from_string("Incident at 1, 2".to_string()),
            incident
        );
    }

    #[test]
    fn test_write_to() {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let location = Location::new("1.0".to_string(), "2.0".to_string());
        let incident = Incident::new(location.clone());

        incident.write_to(&mut cursor).unwrap();
        let buffer = cursor.into_inner();
        assert_eq!(buffer, vec![0, 1, 50, 0, 1, 49]);
    }
}
