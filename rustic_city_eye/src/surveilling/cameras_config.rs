
use std::fs::File;

use serde::{Deserialize, Serialize};

use super::camera::Camera;

#[derive(Debug, Serialize, Deserialize)]
pub struct CamerasConfig{
    pub cameras: Vec<Camera>,
}

impl CamerasConfig {
    
    pub fn new() -> CamerasConfig {
        CamerasConfig {
            cameras: Vec::new(),
        }
    }

    pub fn camera_exists(id: u32) -> bool {
        let path = format!("./src/surveilling/cameras.json");
        if !std::fs::metadata(&path).is_ok() {
            return false;
        }

        let file =  match File::open(path) {
            Ok(file) => file,
            Err(_) => return false,
        };
        
        let cameras_config: CamerasConfig = match serde_json::from_reader(file) {
            Ok(cameras_config) => cameras_config,
            Err(_) => return false,
        };

        for camera in cameras_config.cameras {
            if camera.id == id {
                return true;
            }
        }

        false
    }

    pub fn count_cameras() -> u32 {
        let path = format!("./src/surveilling/cameras.json");
        if !std::fs::metadata(&path).is_ok() {
            return 0;
        }

        let file =  match File::open(path) {
            Ok(file) => file,
            Err(_) => return 0,
        };
        
        let cameras_config: CamerasConfig = match serde_json::from_reader(file) {
            Ok(cameras_config) => cameras_config,
            Err(_) => return 0,
        };

        cameras_config.cameras.len() as u32
    }

    pub fn add_camera_to_json(camera: Camera) -> Result<(), Box<dyn std::error::Error>> {

        let path = format!("./src/surveilling/cameras.json");
        // si el path no existe, lo crea
        if !std::fs::metadata(&path).is_ok() {
            let cameras_config = CamerasConfig::new();
            let json = serde_json::to_string(&cameras_config)?;
            std::fs::write(path.clone(), json)?;
        }
    
        let file = File::open(path.clone())?;
        let mut cameras_config: CamerasConfig = serde_json::from_reader(file)?;
        
        cameras_config.cameras.push(camera);
        let json = serde_json::to_string(&cameras_config)?;
        std::fs::write(path, json)?;


        Ok(())

    }

    pub fn remove_camera_from_file(id: u32) -> Result<(), Box<dyn std::error::Error>> {
        let path = format!("./src/surveilling/cameras.json");
        // si el path no existe retorna ok
        if !CamerasConfig::camera_exists(id) {
            return Ok(());
        }
        std::fs::remove_file(path)?;
        Ok(())
    }

    pub fn get_cameras() -> Vec<Camera> {
        let path = format!("./src/surveilling/cameras.json");
        if !std::fs::metadata(&path).is_ok() {
            return Vec::new();
        }

        let file =  match File::open(path) {
            Ok(file) => file,
            Err(_) => return Vec::new(),
        };
        
        let cameras_config: CamerasConfig = match serde_json::from_reader(file) {
            Ok(cameras_config) => cameras_config,
            Err(_) => return Vec::new(),
        };

        cameras_config.cameras
    }

   


}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surveilling::camera::Camera;
    use crate::utils::location::Location;

    #[test]
    fn test_create_cameras_config_file() {
        let cameras_config = CamerasConfig::new();
        assert_eq!(cameras_config.cameras.len(), 0);
    }

    #[test]
    fn test_add_camera() {
        let location = Location::new(0.0, 0.0);
        let camera = Camera::new(location, 0).unwrap();
        let _ = CamerasConfig::add_camera_to_json(camera);
        assert!(CamerasConfig::camera_exists(0));
    }

    #[test]
    fn test_remove_camera() {
        let location = Location::new(0.0, 0.0);
        let camera = Camera::new(location, 0).unwrap();
        let _ = CamerasConfig::add_camera_to_json(camera);
        assert!(CamerasConfig::camera_exists(0));
        let _ = CamerasConfig::remove_camera_from_file(0);
        assert!(!CamerasConfig::camera_exists(0));
    }

}
