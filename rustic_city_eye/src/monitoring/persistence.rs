use std::fs::File;

use serde::{Deserialize, Serialize};

use crate::{
    drones::{drone_center::DroneCenter, drone_error::DroneError},
    surveilling::camera::Camera,
    utils::location::Location,
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persistence {
    pub cameras: Vec<Camera>,
    pub drone_centers: Vec<(u32, Location, String, String)>,
    pub drones: Vec<(Location, u32)>,
    pub incidents: Vec<Location>,
}

impl Persistence {
    pub fn new() -> Persistence {
        Persistence {
            cameras: Vec::new(),
            drone_centers: Vec::new(),
            drones: Vec::new(),
            incidents: Vec::new(),
        }
    }

    /// Verifica si una cámara con el id dado existe en el archivo cameras.json
    pub fn camera_exists(id: u32) -> bool {
        let path = "./src/monitoring/persistence.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return false;
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return false,
        };

        let p: Persistence = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(_) => return false,
        };

        for camera in p.cameras {
            if camera.id == id {
                return true;
            }
        }

        false
    }

    /// Devuelve la cantidad de cámaras en el archivo cameras.json
    pub fn count_cameras() -> u32 {
        let path = "./src/monitoring/persistence.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return 0;
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return 0,
        };

        let p: Persistence = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(_) => return 0,
        };

        p.cameras.len() as u32
    }

    /// Agrega una cámara al archivo 
    pub fn add_camera_to_file(camera: Camera) -> Result<(), Box<dyn std::error::Error>> {
        let path = "./src/monitoring/persistence.json".to_string();

        if std::fs::metadata(&path).is_err() {
            let p = Persistence::new();
            let json = serde_json::to_string(&p)?;
            std::fs::write(path.clone(), json)?;
        }

        let file = File::open(path.clone())?;
        let mut p: Persistence = serde_json::from_reader(file)?;

        p.cameras.push(camera);
        let json = serde_json::to_string(&p)?;
        std::fs::write(path, json)?;

        println!("Camera added to json");

        Ok(())
    }

    /// Remueve una cámara con el id dado del archivo cameras.json
    pub fn remove_camera_from_file(id: u32) -> Result<(), Box<dyn std::error::Error>> {
        let path = "./src/monitoring/persistence.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return Ok(());
        }

        let file = File::open(path.clone())?;
        let mut p: Persistence = serde_json::from_reader(file)?;

        if let Some(index) = p.cameras.iter().position(|x| x.id == id) {
            p.cameras.remove(index);
            let json = serde_json::to_string(&p)?;
            std::fs::write(path, json)?;
        }

        Ok(())
    }

    /// Devuelve todas las cámaras en el archivo cameras.json
    pub fn get_cameras() -> Vec<Camera> {
        let path = "./src/monitoring/persistence.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return Vec::new();
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Vec::new(),
        };

        let p: Persistence = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        p.cameras
    }

    /// Verifica si un centro de drones con el id dado existe en el archivo drones_central_config.json
    pub fn central_exists(id: u32) -> bool {
        let path = "./src/monitoring/persistence.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return false;
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return false,
        };

        let drones_central_config: Persistence = match serde_json::from_reader(file) {
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

    /// Devuelve la cantidad de centros de drones en el archivo drones_central_config.json
    pub fn count_centrals() -> u32 {
        let path = "./src/monitoring/persistence.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return 0;
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return 0,
        };

        let drones_central_config: Persistence = match serde_json::from_reader(file) {
            Ok(drones_central_config) => drones_central_config,
            Err(_) => return 0,
        };

        drones_central_config.drone_centers.len() as u32
    }

    /// Devuelve los centros de drones en el archivo drones_central_config.json
    pub fn get_centrals() -> Vec<DroneCenter> {
        let path = "./src/monitoring/persistence.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return Vec::new();
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Vec::new(),
        };

        let drones_central_config: Persistence = match serde_json::from_reader(file) {
            Ok(drones_central_config) => drones_central_config,
            Err(_) => return Vec::new(),
        };

        let mut drone_centers = Vec::new();
        for (id, location, config_path, address) in drones_central_config.drone_centers {
            println!("Loaded drone center {}", id);
            let drone_center = DroneCenter::new(id, location, config_path, address);
            drone_centers.push(drone_center);
        }

        drone_centers
    }

    /// Agrega un centro de drones al archivo drones_central_config.json
    pub fn add_central_to_file(
        id: u32,
        location: Location,
        config_path: String,
        address: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = "./src/monitoring/persistence.json".to_string();
        if std::fs::metadata(&path).is_err() {
            let drones_central_config = Persistence::new();
            let json = serde_json::to_string(&drones_central_config)?;
            std::fs::write(path.clone(), json)?;
        }

        let file = File::open(path.clone())?;

        let mut drones_central_config: Persistence = serde_json::from_reader(file)?;

        drones_central_config
            .drone_centers
            .push((id, location, config_path, address));
        let json = serde_json::to_string(&drones_central_config)?;
        std::fs::write(path, json)?;

        Ok(())
    }

    /// Elimina un centro de drones con el id dado del archivo drones_central_config.json
    pub fn remove_central_from_file(id: u32) -> Result<(), DroneError> {
        let path = "./src/monitoring/persistence.json".to_string();
        if !Persistence::central_exists(id) {
            return Ok(());
        }
        let file = File::open(path.clone()).map_err(|_| {
            DroneError::CentralError(
                "Error while opening the drone center persistent file".to_string(),
            )
        })?;
        let mut drones_central_config: Persistence =
            serde_json::from_reader(file).map_err(|_| {
                DroneError::CentralError(
                    "Error while reading the drone center persistent file".to_string(),
                )
            })?;
        if let Some(index) = drones_central_config
            .drone_centers
            .iter()
            .position(|x| x.0 == id)
        {
            drones_central_config.drone_centers.remove(index);
            let json = serde_json::to_string(&drones_central_config).map_err(|_| {
                DroneError::CentralError(
                    "Error while serializing the drone center persistent file".to_string(),
                )
            })?;
            std::fs::write(path, json).map_err(|_| {
                DroneError::CentralError(
                    "Error while writing the drone center persistent file".to_string(),
                )
            })?;
        }
        Ok(())
    }

    /// Verifica si un dron con el id dado existe en el archivo drones_central_config.json
    pub fn drone_exists(id: u32) -> bool {
        let path = "./src/monitoring/persistence.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return false;
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return false,
        };

        let drones_central_config: Persistence = match serde_json::from_reader(file) {
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

    /// Devuelve la cantidad de drones en el archivo drones_central_config.json
    pub fn count_drones() -> u32 {
        let path = "./src/monitoring/persistence.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return 0;
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return 0,
        };

        let drones_central_config: Persistence = match serde_json::from_reader(file) {
            Ok(drones_central_config) => drones_central_config,
            Err(_) => return 0,
        };

        drones_central_config.drones.len() as u32
    }

    /// Devuelve los drones en el archivo drones_central_config.json
    pub fn get_drones() -> Vec<(Location, u32)> {
        let path = "./src/monitoring/persistence.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return Vec::new();
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Vec::new(),
        };

        let drones_central_config: Persistence = match serde_json::from_reader(file) {
            Ok(drones_central_config) => drones_central_config,
            Err(_) => return Vec::new(),
        };

        drones_central_config.drones
    }

    /// Agrega un dron al archivo drones_central_config.json
    pub fn add_drone_to_file(location: Location, id: u32) -> Result<(), DroneError> {
        let path = "./src/monitoring/persistence.json".to_string();
        if std::fs::metadata(&path).is_err() {
            let drones_central_config = Persistence::new();
            let json = serde_json::to_string(&drones_central_config).map_err(|_| {
                DroneError::CentralError(
                    "Error while serializing the drone center persistent file".to_string(),
                )
            })?;
            std::fs::write(path.clone(), json).map_err(|_| {
                DroneError::CentralError(
                    "Error while writing the drone center persistent file".to_string(),
                )
            })?;
        }

        let file = File::open(path.clone()).map_err(|_| {
            DroneError::CentralError(
                "Error while opening the drone center persistent file".to_string(),
            )
        })?;
        let mut drones_central_config: Persistence =
            serde_json::from_reader(file).map_err(|_| {
                DroneError::CentralError(
                    "Error while reading the drone center persistent file".to_string(),
                )
            })?;

        drones_central_config.drones.push((location, id));
        let json = serde_json::to_string(&drones_central_config).map_err(|_| {
            DroneError::CentralError(
                "Error while serializing the drone center persistent file".to_string(),
            )
        })?;
        std::fs::write(path, json).map_err(|_| {
            DroneError::CentralError(
                "Error while writing the drone center persistent file".to_string(),
            )
        })?;

        Ok(())
    }

    /// Elimina un dron con el id dado del archivo drones_central_config.json
    pub fn remove_drone_from_file(id: u32) -> Result<(), DroneError> {
        let path = "./src/monitoring/persistence.json".to_string();
        if !Persistence::drone_exists(id) {
            return Ok(());
        }
        let file = File::open(path.clone()).map_err(|_| {
            DroneError::CentralError(
                "Error while opening the drone center persistent file".to_string(),
            )
        })?;
        let mut drones_central_config: Persistence =
            serde_json::from_reader(file).map_err(|_| {
                DroneError::CentralError(
                    "Error while reading the drone center persistent file".to_string(),
                )
            })?;
        if let Some(index) = drones_central_config.drones.iter().position(|x| x.1 == id) {
            drones_central_config.drones.remove(index);
            let json = serde_json::to_string(&drones_central_config).map_err(|_| {
                DroneError::CentralError(
                    "Error while serializing the drone center persistent file".to_string(),
                )
            })?;
            std::fs::write(path, json).map_err(|_| {
                DroneError::CentralError(
                    "Error while writing the drone center persistent file".to_string(),
                )
            })?;
        }
        Ok(())
    }

    pub fn count_incidents() -> u32 {
        let path = "./src/monitoring/persistence.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return 0;
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return 0,
        };

        let p: Persistence = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(_) => return 0,
        };

        p.incidents.len() as u32
    }

    pub fn get_incidents() -> Vec<Location> {
        let path = "./src/monitoring/persistence.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return Vec::new();
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Vec::new(),
        };

        let p: Persistence = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        p.incidents        
    }

    pub fn add_incident_to_file(location: Location) -> Result<(), Box<dyn std::error::Error>> {
        let path = "./src/monitoring/persistence.json".to_string();
        if std::fs::metadata(&path).is_err() {
            let p = Persistence::new();
            let json = match serde_json::to_string(&p) {
                Ok(json) => json,
                Err(_) =>{
                    println!("Error while serializing the persistence file");
                    return Ok(());
                }
            };


            let _ = match std::fs::write(path.clone(), json){
                Ok(_) => (),
                Err(_) => {
                    println!("Error while writing the persistence file");
                    return Ok(());
                }
            };
            println!("File created");
        }

        let file = match File::open(path.clone()){
            Ok(file) => file,
            Err(_) => {
                println!("Error while opening the persistence file");
                return Ok(());
            }
        };

        let mut p: Persistence = match serde_json::from_reader(file){
            Ok(p) => p,
            Err(_) => {
                println!("Error while reading the persistence file");
                return Ok(());
            }
        };
        println!("File opened");

        p.incidents.push(location);

        let json = match serde_json::to_string(&p){
            Ok(json) => json,
            Err(_) => {
                println!("Error while serializing the persistence file");
                return Ok(());
            }
        };
        let _ = match std::fs::write(path, json){
            Ok(_) => (),
            Err(_) => {
                println!("Error while writing the persistence file");
                return Ok(());
            }
        };

        println!("Incident added to file");
        Ok(())
    }

    pub fn remove_incident_from_file(location: Location) -> Result<(), Box<dyn std::error::Error>> {
        let path = "./src/monitoring/persistence.json".to_string();
        if std::fs::metadata(&path).is_err() {
            return Ok(());
        }

        let file = File::open(path.clone())?;
        let mut p: Persistence = serde_json::from_reader(file)?;

        if let Some(index) = p.incidents.iter().position(|x| *x == location) { 
            p.incidents.remove(index);
            let json = serde_json::to_string(&p)?;
            std::fs::write(path, json)?;
        }

        Ok(())
    }
}

impl Default for Persistence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    

    
}
