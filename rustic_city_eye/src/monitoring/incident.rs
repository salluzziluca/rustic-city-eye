use crate::surveilling::location::Location;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Incident {
    location: Location
}

impl Incident {
    pub fn new(location: Location) -> Incident {
        Self { location }
    }
}