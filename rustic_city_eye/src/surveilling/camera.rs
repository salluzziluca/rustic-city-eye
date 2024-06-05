use crate::utils::location::Location;

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new_camera() {
        let location = Location::new(1.0, 2.0);
        let camera = Camera::new(location.clone());
        assert_eq!(camera.get_location(), location);
    }
}
