use crate::{helpers::location::Location, mqtt::writer::write_string};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Incident {
    location: Location,
}

impl Incident {
    pub fn new(location: Location) -> Incident {
        Self { location }
    }

    pub fn parse_to_string(&self) -> String {
        format!("Incident at {}", self.location.parse_to_string())
    }

    pub fn from_string(incident: String) -> Incident {
        let location = incident.split(" at ").collect::<Vec<&str>>()[1];
        Incident {
            location: Location::new(
                location.split(", ").collect::<Vec<&str>>()[0].to_string(),
                location.split(", ").collect::<Vec<&str>>()[1].to_string(),
            ),
        }
    }

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
