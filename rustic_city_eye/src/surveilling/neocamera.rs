use crate::utils::location::Location;

#[derive(Debug)]
pub struct Camera {
    location: Location,
    camera_id: usize,
    state: bool,
    operation_radius: f64
}

impl Camera {
    pub fn new(location: Location, camera_id: usize, operation_radius: f64) -> Camera {
        Camera {
            location,
            camera_id,
            state: false,
            operation_radius
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

    pub fn detect_incident(&mut self, location: Location) {
        let distance = self.location.distance(location);

        if distance <= self.operation_radius {
            self.state = true;
        }
    }

    pub fn get_state(&self) -> bool {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_camera_creation_ok() {
        let location = Location::new(1.2, 2.1);
        let camera_id = 1;

        let _camera = Camera::new(location, camera_id, 10.0);
    }

    #[test]
    fn test_02_camera_at_location_ok() {
        let location = Location::new(1.2, 2.1);
        let wrong_location = Location::new(3.4, 4.3);
        let camera_id = 1;
        let camera = Camera::new(location.clone(), camera_id, 10.0);

        assert!(camera.is_at_location(location));
        assert!(!camera.is_at_location(wrong_location));
    }

    #[test]
    fn test_03_camera_detects_incident_and_change_state_ok() {
        let location = Location::new(1.2, 2.1);
        let incident_location = Location::new(3.4, 4.3);
        let camera_id = 1;
        let mut camera = Camera::new(location.clone(), camera_id, 10.0);
        
        camera.detect_incident(incident_location);

        assert_eq!(camera.get_state(), true);
    }
}
