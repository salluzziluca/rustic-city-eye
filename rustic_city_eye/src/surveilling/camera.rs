use crate::utils::location::Location;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Camera {
    location: Location,
    id: u32,
    sleep_mode: bool,
}

impl Camera {
    pub fn new(location: Location, id: u32) -> Camera {
        Self {
            location,
            id,
            sleep_mode: true,
        }
    }

    pub fn get_location(&self) -> Location {
        self.location.clone()
    }
    pub fn get_id(&self) -> u32 {
        self.id
    }
    pub fn get_sleep_mode(&self) -> bool {
        self.sleep_mode
    }
    pub fn set_sleep_mode(&mut self, sleep_mode: bool) {
        self.sleep_mode = sleep_mode;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new_camera() {
        let location = Location::new(1.0, 2.0);
        let camera = Camera::new(location.clone(), 1);
        assert_eq!(camera.get_location(), location);
        assert_eq!(camera.get_id(), 1);
    }
}
