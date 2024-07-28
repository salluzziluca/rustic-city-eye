use serde::{Deserialize, Serialize};

use super::camera::Camera;

#[derive(Debug, Serialize, Deserialize)]
pub struct CamerasConfig {
    pub cameras: Vec<Camera>,
}

impl CamerasConfig {
    pub fn new() -> CamerasConfig {
        CamerasConfig {
            cameras: Vec::new(),
        }
    }

    pub fn add_camera(&mut self, camera: Camera) {
        if !self.cameras.contains(&camera) {
            self.cameras.push(camera);
        }
    }
   


}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::surveilling::camera::Camera;
    use crate::utils::location::Location;

    #[test]
    fn test_new_cameras_config() {
        let cameras_config = CamerasConfig::new();
        assert_eq!(cameras_config.cameras.len(), 0);
    }

    #[test]
    fn test_add_camera() {
        let mut cameras_config = CamerasConfig::new();
        let location = Location::new(0.0, 0.0);
        let camera = Camera::new(location, 0).unwrap();
        cameras_config.add_camera(camera.clone());
        assert_eq!(cameras_config.cameras.len(), 1);
        cameras_config.add_camera(camera.clone());
        assert_eq!(cameras_config.cameras.len(), 1);
    }
}
