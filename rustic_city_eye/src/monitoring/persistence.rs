use std::fs::File;

use serde::{Deserialize, Serialize};

use crate::{
    drones::{drone_center::DroneCenter, drone_error::DroneError},
    surveilling::camera::Camera,
    utils::location::Location,
};

const PERSISTENCE_FILE: &str = "./src/monitoring/persistence.json";

/// Estructura que representa la persistencia de la información de las cámaras, drones y centros de drones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persistence {
    /// Vector de cámaras
    pub cameras: Vec<Camera>,
    /// Vector con la información de los centros de drones
    pub drone_centers: Vec<(u32, Location, String, String)>,
    /// Vector con la información de los drones
    pub drones: Vec<(Location, u32)>,
    /// Vector con la location de los incidentes
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

    /// Cuenta la cantidad de elementos de un tipo en el archivo
    pub fn count_element(element: String) -> u32 {
        if std::fs::metadata(PERSISTENCE_FILE).is_err() {
            return 0;
        }

        let file = match File::open(PERSISTENCE_FILE) {
            Ok(file) => file,
            Err(_) => return 0,
        };

        let p: Persistence = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(_) => return 0,
        };

        match element.as_str() {
            "cameras" => p.cameras.len() as u32,
            "drones" => p.drones.len() as u32,
            "drone_centers" => p.drone_centers.len() as u32,
            "incidents" => p.incidents.len() as u32,
            _ => 0,
        }
    }

    

    /// Verifica si una cámara con el id dado existe en el archivo
    pub fn camera_exists(id: u32) -> bool {
        if std::fs::metadata(PERSISTENCE_FILE).is_err() {
            return false;
        }

        let file = match File::open(PERSISTENCE_FILE) {
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

    /// Agrega una cámara al archivo 
    pub fn add_camera_to_file(camera: Camera) -> Result<(), Box<dyn std::error::Error>> {
        if std::fs::metadata(PERSISTENCE_FILE).is_err() {
            let p = Persistence::new();
            let json = serde_json::to_string(&p)?;
            std::fs::write(PERSISTENCE_FILE, json)?;
        }

        let file = File::open(PERSISTENCE_FILE)?;
        let mut p: Persistence = serde_json::from_reader(file)?;

        p.cameras.push(camera);
        let json = serde_json::to_string(&p)?;
        std::fs::write(PERSISTENCE_FILE, json)?;

        println!("Camera added to json");

        Ok(())
    }

    /// Remueve una cámara con el id dado del archivo
    pub fn remove_camera_from_file(id: u32) -> Result<(), Box<dyn std::error::Error>> {
        if std::fs::metadata(PERSISTENCE_FILE).is_err() {
            return Ok(());
        }

        let file = File::open(PERSISTENCE_FILE)?;
        let mut p: Persistence = serde_json::from_reader(file)?;

        if let Some(index) = p.cameras.iter().position(|x| x.id == id) {
            p.cameras.remove(index);
            let json = serde_json::to_string(&p)?;
            std::fs::write(PERSISTENCE_FILE, json)?;
        }

        Ok(())
    }

    /// Devuelve todas las cámaras en el archivo
    pub fn get_cameras() -> Vec<Camera> {
        if std::fs::metadata(PERSISTENCE_FILE).is_err() {
            return Vec::new();
        }

        let file = match File::open(PERSISTENCE_FILE) {
            Ok(file) => file,
            Err(_) => return Vec::new(),
        };

        let p: Persistence = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        p.cameras
    }

    /// Verifica si un centro de drones con el id dado existe en el archivo
    pub fn central_exists(id: u32) -> bool {
        if std::fs::metadata(PERSISTENCE_FILE).is_err() {
            return false;
        }

        let file = match File::open(PERSISTENCE_FILE) {
            Ok(file) => file,
            Err(_) => return false,
        };

        let p: Persistence = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(_) => return false,
        };

        for (central_id, _, _, _) in p.drone_centers {
            if central_id == id {
                return true;
            }
        }

        false
    }


    /// Devuelve los centros de drones en el archivo
    pub fn get_centrals() -> Vec<DroneCenter> {
        if std::fs::metadata(PERSISTENCE_FILE).is_err() {
            return Vec::new();
        }

        let file = match File::open(PERSISTENCE_FILE) {
            Ok(file) => file,
            Err(_) => return Vec::new(),
        };

        let p: Persistence = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        let mut drone_centers = Vec::new();
        for (id, location, drone_config_path, address) in p.drone_centers {
            println!("Loaded drone center {}", id);
            let drone_center = DroneCenter::new(id, location, drone_config_path, address);
            drone_centers.push(drone_center);
        }

        drone_centers
    }

    /// Agrega un centro de drones al archivo
    pub fn add_central_to_file(
        id: u32,
        location: Location,
        drone_config_path: String,
        address: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if std::fs::metadata(PERSISTENCE_FILE).is_err() {
            let p = Persistence::new();
            let json = serde_json::to_string(&p)?;
            std::fs::write(PERSISTENCE_FILE, json)?;
        }

        let file = File::open(PERSISTENCE_FILE)?;

        let mut p: Persistence = serde_json::from_reader(file)?;

        p
            .drone_centers
            .push((id, location, drone_config_path, address));
        let json = serde_json::to_string(&p)?;
        std::fs::write(PERSISTENCE_FILE, json)?;

        Ok(())
    }

    /// Elimina un centro de drones con el id dado del archivo
    pub fn remove_central_from_file(id: u32) -> Result<(), DroneError> {
        if !Persistence::central_exists(id) {
            return Ok(());
        }
        let file = File::open(PERSISTENCE_FILE).map_err(|_| {
            DroneError::CentralError(
                "Error while opening the drone center persistent file".to_string(),
            )
        })?;
        let mut p: Persistence =
            serde_json::from_reader(file).map_err(|_| {
                DroneError::CentralError(
                    "Error while reading the drone center persistent file".to_string(),
                )
            })?;
        if let Some(index) = p
            .drone_centers
            .iter()
            .position(|x| x.0 == id)
        {
            p.drone_centers.remove(index);
            let json = serde_json::to_string(&p).map_err(|_| {
                DroneError::CentralError(
                    "Error while serializing the drone center persistent file".to_string(),
                )
            })?;
            std::fs::write(PERSISTENCE_FILE, json).map_err(|_| {
                DroneError::CentralError(
                    "Error while writing the drone center persistent file".to_string(),
                )
            })?;
        }
        Ok(())
    }

    /// Verifica si un dron con el id dado existe en el archivo
    pub fn drone_exists(id: u32) -> bool {
        if std::fs::metadata(PERSISTENCE_FILE).is_err() {
            return false;
        }

        let file = match File::open(PERSISTENCE_FILE) {
            Ok(file) => file,
            Err(_) => return false,
        };

        let p: Persistence = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(_) => return false,
        };

        for (central_id, _, _, _) in p.drone_centers {
            if central_id == id {
                return true;
            }
        }

        false
    }

    /// Devuelve los drones en el archivo
    pub fn get_drones() -> Vec<(Location, u32)> {
        if std::fs::metadata(PERSISTENCE_FILE).is_err() {
            return Vec::new();
        }

        let file = match File::open(PERSISTENCE_FILE) {
            Ok(file) => file,
            Err(_) => return Vec::new(),
        };

        let p: Persistence = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        p.drones
    }

    /// Agrega un dron al archivo
    pub fn add_drone_to_file(location: Location, id: u32) -> Result<(), DroneError> {
        if std::fs::metadata(PERSISTENCE_FILE).is_err() {
            let p = Persistence::new();
            let json = serde_json::to_string(&p).map_err(|_| {
                DroneError::CentralError(
                    "Error while serializing the drone center persistent file".to_string(),
                )
            })?;
            std::fs::write(PERSISTENCE_FILE, json).map_err(|_| {
                DroneError::CentralError(
                    "Error while writing the drone center persistent file".to_string(),
                )
            })?;
        }

        let file = File::open(PERSISTENCE_FILE).map_err(|_| {
            DroneError::CentralError(
                "Error while opening the drone center persistent file".to_string(),
            )
        })?;
        let mut p: Persistence =
            serde_json::from_reader(file).map_err(|_| {
                DroneError::CentralError(
                    "Error while reading the drone center persistent file".to_string(),
                )
            })?;

        p.drones.push((location, id));
        let json = serde_json::to_string(&p).map_err(|_| {
            DroneError::CentralError(
                "Error while serializing the drone center persistent file".to_string(),
            )
        })?;
        std::fs::write(PERSISTENCE_FILE, json).map_err(|_| {
            DroneError::CentralError(
                "Error while writing the drone center persistent file".to_string(),
            )
        })?;

        Ok(())
    }

    /// Elimina un dron con el id dado del archivo
    pub fn remove_drone_from_file(id: u32) -> Result<(), DroneError> {
        if !Persistence::drone_exists(id) {
            return Ok(());
        }
        let file = File::open(PERSISTENCE_FILE).map_err(|_| {
            DroneError::CentralError(
                "Error while opening the drone center persistent file".to_string(),
            )
        })?;
        let mut p: Persistence =
            serde_json::from_reader(file).map_err(|_| {
                DroneError::CentralError(
                    "Error while reading the drone center persistent file".to_string(),
                )
            })?;
        if let Some(index) = p.drones.iter().position(|x| x.1 == id) {
            p.drones.remove(index);
            let json = serde_json::to_string(&p).map_err(|_| {
                DroneError::CentralError(
                    "Error while serializing the drone center persistent file".to_string(),
                )
            })?;
            std::fs::write(PERSISTENCE_FILE, json).map_err(|_| {
                DroneError::CentralError(
                    "Error while writing the drone center persistent file".to_string(),
                )
            })?;
        }
        Ok(())
    }

    /// Devuelve los incidentes en el archivo
    pub fn get_incidents() -> Vec<Location> {
        if std::fs::metadata(PERSISTENCE_FILE).is_err() {
            return Vec::new();
        }

        let file = match File::open(PERSISTENCE_FILE) {
            Ok(file) => file,
            Err(_) => return Vec::new(),
        };

        let p: Persistence = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        p.incidents        
    }

    /// Agrega un incidente al archivo
    pub fn add_incident_to_file(location: Location) -> Result<(), Box<dyn std::error::Error>> {
        if std::fs::metadata(PERSISTENCE_FILE).is_err() {
            let p = Persistence::new();
            let json = serde_json::to_string(&p)?;
            std::fs::write(PERSISTENCE_FILE, json)?;
        }

        let file = File::open(PERSISTENCE_FILE)?;

        let mut p: Persistence =serde_json::from_reader(file)?;

        p.incidents.push(location);

        let json = serde_json::to_string(&p)?;
        std::fs::write(PERSISTENCE_FILE, json)?;

        Ok(())
    }

    /// Elimina un incidente con la location dada del archivo
    pub fn remove_incident_from_file(location: Location) -> Result<(), Box<dyn std::error::Error>> {
        if std::fs::metadata(PERSISTENCE_FILE).is_err() {
            return Ok(());
        }

        let file = File::open(PERSISTENCE_FILE)?;
        let mut p: Persistence = serde_json::from_reader(file)?;

        if let Some(index) = p.incidents.iter().position(|x| *x == location) { 
            p.incidents.remove(index);
            let json = serde_json::to_string(&p)?;
            std::fs::write(PERSISTENCE_FILE, json)?;
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
