
use std::fs::File;

use serde::{Deserialize, Serialize};

use crate::utils::location::Location;

use super::camera::Camera;

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraConfig {
    pub location: Location,
    pub id: u32,
    pub sleep_mode: bool,
}

impl CameraConfig {
    pub fn new(location: Location, id: u32) -> CameraConfig {
        CameraConfig {
            location,
            id,
            sleep_mode: true,
        }
    }

    pub fn camera_exists(id: u32) -> bool {
        let path = format!("./src/surveilling/cameras/{}.json", id);
        let file = File::open(path);
        match file {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn create_camera_file(camera: Camera) -> Result<(), Box<dyn std::error::Error>> {
        let camera_config = CameraConfig::new(camera.location.clone(), camera.id.clone());
        let json = serde_json::to_string(&camera_config)?;
        let path = format!("./src/surveilling/cameras/{}.json", camera.id);

        std::fs::write(path, json)?;
        Ok(())
       
    }

    pub fn add_camera_to_json(camera: Camera) -> Result<(), Box<dyn std::error::Error>> {
        let path = format!("./src/surveilling/cameras/{}.json", camera.id);
        if !CameraConfig::camera_exists(camera.id.clone()) {
            let _ = CameraConfig::create_camera_file(camera.clone());
        }

        let file = File::open(path.clone())?;
        let mut camera_config: CameraConfig = serde_json::from_reader(file)?;
        camera_config.sleep_mode = camera.sleep_mode;
        camera_config.location = camera.location;
        let json = serde_json::to_string(&camera_config)?;
        std::fs::write(path, json)?;

        Ok(())

    }


    pub fn remove_camera(id: u32) -> Result<(), Box<dyn std::error::Error>> {
        let path = format!("./src/surveilling/cameras/{}.json", id);
        // si el path no existe retorna ok
        if !CameraConfig::camera_exists(id) {
            return Ok(());
        }
        std::fs::remove_file(path)?;
        Ok(())
    }

    pub fn get_camera_config(id: u32) -> Result<CameraConfig, Box<dyn std::error::Error>> {
        let path = format!("./src/surveilling/cameras/{}.json", id);
        let file = File::open(path)?;
        let camera_config: CameraConfig = serde_json::from_reader(file)?;
        Ok(camera_config)
    }





   


}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surveilling::camera::Camera;
    use crate::utils::location::Location;

    #[test]
    fn test_new_camera_config() {
        let location = Location::new(0.0, 0.0);
        let camera_config = CameraConfig::new(location, 0);
        assert_eq!(camera_config.location, location);
        assert_eq!(camera_config.id, 0);
        assert_eq!(camera_config.sleep_mode, true);

        // eliminar archivos creados
    }

    #[test]
    fn test_add_camera() {
        let location = Location::new(0.0, 0.0);
        let camera = Camera::new(location.clone(), 0).unwrap();
        let _ = CameraConfig::add_camera_to_json(camera.clone());
        let camera_config = CameraConfig::new(location.clone(), 0);
        let path = format!("./src/surveilling/cameras/{}.json", camera.id);
        let file = File::open(path).unwrap();
        let camera_config_read: CameraConfig = serde_json::from_reader(file).unwrap();
        
        assert_eq!(camera_config.id, camera_config_read.id);
        assert_eq!(camera_config.location, camera_config_read.location);
        assert_eq!(camera_config.sleep_mode, camera_config_read.sleep_mode);

        // eliminar archivos creados
        let _ = CameraConfig::remove_camera(0);
    }


}
