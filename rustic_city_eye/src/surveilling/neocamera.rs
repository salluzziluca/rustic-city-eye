use crate::utils::location::Location;

#[derive(Debug)]
pub struct Camera {
    location: Location,
    camera_id: usize,
    state: bool,
}

impl Camera {
    pub fn new(location: Location, camera_id: usize) -> Camera {
        Camera {
            location,
            camera_id,
            state: false,
        }
    }

    /// Dada una location determinada, se devuelve true
    /// si la camara esta colocada en esa location, y se devuelve false
    /// en caso contrario.
    pub fn is_at_location(&self, location: Location) -> bool {
        if self.location == location {
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_camera_creation_ok() {
        let location = Location::new(1.2, 2.1);
        let camera_id = 1;

        let _camera = Camera::new(location, camera_id);
    }

    #[test]
    fn test_02_camera_at_location_ok() {
        let location = Location::new(1.2, 2.1);
        let wrong_location = Location::new(3.4, 4.3);
        let camera_id = 1;
        let camera = Camera::new(location.clone(), camera_id);

        assert!(camera.is_at_location(location));
        assert!(!camera.is_at_location(wrong_location));
    }
}
