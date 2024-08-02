use std::fs::File;

use serde::{Deserialize, Serialize};

use crate::utils::location::Location;

use super::{drone_center::DroneCenter, drone_error::DroneError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DronesCentralConfig {
    pub drones: Vec<(Location, u32)>,
    pub drone_centers: Vec<(u32, Location, String, String)>,
}

impl DronesCentralConfig{
    pub fn new() -> DronesCentralConfig {
        DronesCentralConfig {
            drones: Vec::new(),
            drone_centers: Vec::new(),
        }
    }

    pub fn central_exists(id: u32) -> bool {
        let path = "./src/drones/drones_central_config.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return false;
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return false,
        };

        let drones_central_config: DronesCentralConfig = match serde_json::from_reader(file) {
            Ok(drones_central_config) => drones_central_config,
            Err(_) => return false,
        };

        for (central_id, _, _, _) in drones_central_config.drone_centers {
            if central_id == id {
                return true;
            }
        }

        false
    }

    pub fn count_centrals() -> u32 {
        let path = "./src/drones/drones_central_config.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return 0;
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return 0,
        };

        let drones_central_config: DronesCentralConfig = match serde_json::from_reader(file) {
            Ok(drones_central_config) => drones_central_config,
            Err(_) => return 0,
        };

        drones_central_config.drone_centers.len() as u32
    }

    pub fn get_centrals() -> Vec<DroneCenter> {
        let path = "./src/drones/drones_central_config.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return Vec::new();
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Vec::new(),
        };

        let drones_central_config: DronesCentralConfig = match serde_json::from_reader(file) {
            Ok(drones_central_config) => drones_central_config,
            Err(_) => return Vec::new(),
        };

        let mut drone_centers = Vec::new();
        for (id, location, config_path, address) in drones_central_config.drone_centers {
            let drone_center = DroneCenter::new(id, location, config_path, address);
            drone_centers.push(drone_center);
        }

        drone_centers
    }

    pub fn add_central_to_json(id: u32, location: Location, config_path: String, address: String) -> Result<(), DroneError> {
        let path = "./src/drones/drones_central_config.json".to_string();
        if std::fs::metadata(&path).is_err() {
            let drones_central_config = DronesCentralConfig::new();
            let json = serde_json::to_string(&drones_central_config).map_err(|_| DroneError::CentralError("Error while serializing the drone center persistent file".to_string()))?;
            std::fs::write(path.clone(), json).map_err(|_| DroneError::CentralError("Error while writing the drone center persistent file".to_string()))?;
        }

        let file = File::open(path.clone()).map_err(|_| DroneError::CentralError("Error while opening the drone center persistent file".to_string()))?;
        let mut drones_central_config: DronesCentralConfig = serde_json::from_reader(file).map_err(|_| DroneError::CentralError("Error while reading the drone center persistent file".to_string()))?;

        drones_central_config.drone_centers.push((id, location, config_path, address));
        let json = serde_json::to_string(&drones_central_config).map_err(|_| DroneError::CentralError("Error while serializing the drone center persistent file".to_string()))?;
        std::fs::write(path, json).map_err(|_| DroneError::CentralError("Error while writing the drone center persistent file".to_string()))?;

        Ok(())
    }

    pub fn remove_central_from_json(id: u32) -> Result<(), DroneError> {
        let path = "./src/drones/drones_central_config.json".to_string();
        if !DronesCentralConfig::central_exists(id) {
            return Ok(());
        }
        std::fs::remove_file(path).map_err(|_| DroneError::CentralError("Error while removing the drone center persistent file".to_string()))?;
        Ok(())
    }

    pub fn drone_exists(id: u32) -> bool {
        let path = "./src/drones/drones_central_config.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return false;
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return false,
        };

        let drones_central_config: DronesCentralConfig = match serde_json::from_reader(file) {
            Ok(drones_central_config) => drones_central_config,
            Err(_) => return false,
        };

        for (central_id, _, _, _) in drones_central_config.drone_centers {
            if central_id == id {
                return true;
            }
        }

        false
    }

    pub fn count_drones() -> u32 {
        let path = "./src/drones/drones_central_config.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return 0;
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return 0,
        };

        let drones_central_config: DronesCentralConfig = match serde_json::from_reader(file) {
            Ok(drones_central_config) => drones_central_config,
            Err(_) => return 0,
        };

        drones_central_config.drones.len() as u32
    }

    pub fn get_drones() -> Vec<(Location, u32)> {
        let path = "./src/drones/drones_central_config.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return Vec::new();
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Vec::new(),
        };

        let drones_central_config: DronesCentralConfig = match serde_json::from_reader(file) {
            Ok(drones_central_config) => drones_central_config,
            Err(_) => return Vec::new(),
        };

        drones_central_config.drones
    }

    pub fn add_drone_to_json(location: Location, id: u32) -> Result<(), DroneError> {
        let path = "./src/drones/drones_central_config.json".to_string();
        if std::fs::metadata(&path).is_err() {
            let drones_central_config = DronesCentralConfig::new();
            let json = serde_json::to_string(&drones_central_config).map_err(|_| DroneError::CentralError("Error while serializing the drone center persistent file".to_string()))?;
            std::fs::write(path.clone(), json).map_err(|_| DroneError::CentralError("Error while writing the drone center persistent file".to_string()))?;
        }

        let file = File::open(path.clone()).map_err(|_| DroneError::CentralError("Error while opening the drone center persistent file".to_string()))?;
        let mut drones_central_config: DronesCentralConfig = serde_json::from_reader(file).map_err(|_| DroneError::CentralError("Error while reading the drone center persistent file".to_string()))?;

        drones_central_config.drones.push((location, id));
        let json = serde_json::to_string(&drones_central_config).map_err(|_| DroneError::CentralError("Error while serializing the drone center persistent file".to_string()))?;
        std::fs::write(path, json).map_err(|_| DroneError::CentralError("Error while writing the drone center persistent file".to_string()))?;

        Ok(())
    }

    pub fn remove_drone_from_json(id: u32) -> Result<(), DroneError> {
        let path = "./src/drones/drones_central_config.json".to_string();
        if !DronesCentralConfig::drone_exists(id) {
            return Ok(());
        }
        std::fs::remove_file(path).map_err(|_| DroneError::CentralError("Error while removing the drone center persistent file".to_string()))?;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drones_central_config_central_create() {
        let drone_center = DronesCentralConfig::new();
        assert_eq!(drone_center.drone_centers.len(), 0);
    }

    #[test]
    fn test_drones_central_config_add_central() {
        let path = "./src/drones/drones_central_config.json".to_string();
        if std::fs::metadata(&path).is_ok() {
            std::fs::remove_file(&path).unwrap();
        }

        let location = Location::new(1.0, 1.0);
        let _ = DronesCentralConfig::add_central_to_json(1, location, "config_path".to_string(), "address".to_string());
        assert!(DronesCentralConfig::central_exists(1));
        std::fs::remove_file(&path)
            .expect("Error while removing the drone center persistent file");

    }

    #[test]
    fn test_drones_central_config_get_centrals() {
        let drone_centers = DronesCentralConfig::get_centrals();
        assert_eq!(drone_centers.len(), 0);
        DronesCentralConfig::add_central_to_json(1, Location::new(1.0, 1.0), "config_path".to_string(), "address".to_string()).unwrap();
        let drone_centers = DronesCentralConfig::get_centrals();
        assert_eq!(drone_centers.len(), 1);
        std::fs::remove_file("./src/drones/drones_central_config.json").unwrap();
    }

    #[test]
    fn test_drones_config_add_drone(){
        let path = "./src/drones/drones_central_config.json".to_string();
        if std::fs::metadata(&path).is_ok() {
            std::fs::remove_file(&path).unwrap();
        
        let location = Location::new(1.0, 1.0);
        let _ = DronesCentralConfig::add_drone_to_json(location, 1);
        assert!(DronesCentralConfig::drone_exists(1));
        std::fs::remove_file(&path)
            .expect("Error while removing the drone center persistent file");
        }
    }
}