use crate::surveilling::location::Location;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Incident {
    location: Location,
}

impl Incident {
    pub fn new(location: Location) -> Incident {
        Self { location }
    }

    pub fn to_string(&self) -> String {
        format!("Incident at {}", self.location.to_string())
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
}
