use crate::surveilling::location::Location;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Camera {
    location: Location,
}

impl Camera {
    pub fn new(location: Location) -> Camera {
        Self { location }
    }

    pub fn get_location(&self) -> Location {
        self.location.clone()
    }
}
