use crate::surveilling::location::Location;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Camera {
    location: Location,
}

impl Camera {
    pub fn new(location: Location) -> Camera {
        Self { location }
    }
}
