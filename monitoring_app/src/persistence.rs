use std::{fs::File, path::PathBuf};

use drones::{drone_center::DroneCenter, drone_error::DroneError};
use serde::{Deserialize, Serialize};

use utils::{camera::Camera, location::Location};

const PERSISTENCE_FILE: &str = "persistence/persistence.json";
const TEST_PERSISTENCE_FILE: &str = "persistence/test_persistence.json";

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

    fn get_filepath() -> String {
        let project_dir = env!("CARGO_MANIFEST_DIR");
        let mut path = PathBuf::from(project_dir);
        if cfg!(test) {
            path.push(TEST_PERSISTENCE_FILE);
        } else {
            path.push(PERSISTENCE_FILE);
        }
        path.to_str().unwrap().to_owned()
    }

    /// Cuenta la cantidad de elementos de un tipo en el archivo de persistencia
    pub fn count_element(element: String) -> u32 {
        let file_path = Self::get_filepath();
        if std::fs::metadata(file_path.clone().clone()).is_err() {
            return 0;
        }

        let file = match File::open(file_path) {
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

    /// Verifica si una cámara con el id dado existe en el archivo de persistencia
    pub fn camera_exists(id: u32) -> bool {
        let file_path = Self::get_filepath();
        if std::fs::metadata(file_path.clone()).is_err() {
            return false;
        }

        let file = match File::open(file_path) {
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

    /// Agrega una cámara al archivo de persistencia
    pub fn add_camera_to_file(camera: Camera) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = Self::get_filepath();
        if std::fs::metadata(file_path.clone()).is_err() {
            let p = Persistence::new();
            let json = serde_json::to_string(&p)?;
            std::fs::write(file_path.clone(), json)?;
        }

        let file = File::open(file_path.clone())?;
        let mut p: Persistence = serde_json::from_reader(file)?;

        p.cameras.push(camera);
        let json = serde_json::to_string(&p)?;
        std::fs::write(file_path, json)?;

        println!("Camera added to json");

        Ok(())
    }

    /// Remueve una cámara con el id dado del archivo de persistencia
    pub fn remove_camera_from_file(id: u32) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = Self::get_filepath();
        if std::fs::metadata(file_path.clone()).is_err() {
            return Ok(());
        }

        let file = File::open(file_path.clone())?;
        let mut p: Persistence = serde_json::from_reader(file)?;

        if let Some(index) = p.cameras.iter().position(|x| x.id == id) {
            p.cameras.remove(index);
            let json = serde_json::to_string(&p)?;
            std::fs::write(file_path, json)?;
        }

        Ok(())
    }

    /// Devuelve todas las cámaras en el archivo de persistencia
    pub fn get_cameras() -> Vec<Camera> {
        let file_path = Self::get_filepath();
        if std::fs::metadata(file_path.clone()).is_err() {
            return Vec::new();
        }

        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => return Vec::new(),
        };

        let p: Persistence = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        p.cameras
    }

    /// Verifica si un centro de drones con el id dado existe en el archivo de persistencia
    pub fn central_exists(id: u32) -> bool {
        let file_path = Self::get_filepath();
        if std::fs::metadata(file_path.clone()).is_err() {
            return false;
        }

        let file = match File::open(file_path) {
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

    /// Devuelve los centros de drones en el archivo de persistencia
    pub fn get_centrals() -> Vec<DroneCenter> {
        let file_path = Self::get_filepath();
        if std::fs::metadata(file_path.clone()).is_err() {
            return Vec::new();
        }

        let file = match File::open(file_path) {
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

    /// Agrega un centro de drones al archivo de persistencia
    pub fn add_center_to_file(
        id: u32,
        location: Location,
        drone_config_path: String,
        address: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = Self::get_filepath();
        if std::fs::metadata(file_path.clone()).is_err() {
            let p = Persistence::new();
            let json = serde_json::to_string(&p)?;
            std::fs::write(file_path.clone(), json)?;
        }

        let file = File::open(file_path.clone())?;

        let mut p: Persistence = serde_json::from_reader(file)?;

        p.drone_centers
            .push((id, location, drone_config_path, address));
        let json = serde_json::to_string(&p)?;
        std::fs::write(file_path, json)?;

        Ok(())
    }

    /// Elimina un centro de drones con el id dado del archivo de persistencia
    pub fn remove_central_from_file(id: u32) -> Result<(), DroneError> {
        let file_path = Self::get_filepath();
        if !Persistence::central_exists(id) {
            return Ok(());
        }
        let file = File::open(file_path.clone()).map_err(|_| {
            DroneError::CentralError(
                "Error while opening the drone center persistent file".to_string(),
            )
        })?;
        let mut p: Persistence = serde_json::from_reader(file).map_err(|_| {
            DroneError::CentralError(
                "Error while reading the drone center persistent file".to_string(),
            )
        })?;
        if let Some(index) = p.drone_centers.iter().position(|x| x.0 == id) {
            p.drone_centers.remove(index);
            let json = serde_json::to_string(&p).map_err(|_| {
                DroneError::CentralError(
                    "Error while serializing the drone center persistent file".to_string(),
                )
            })?;
            std::fs::write(file_path, json).map_err(|_| {
                DroneError::CentralError(
                    "Error while writing the drone center persistent file".to_string(),
                )
            })?;
        }
        Ok(())
    }

    /// Verifica si un dron con el id dado existe en el archivo de persistencia
    pub fn drone_exists(id: u32) -> bool {
        let file_path = Self::get_filepath();
        if std::fs::metadata(file_path.clone()).is_err() {
            return false;
        }

        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => return false,
        };

        let p: Persistence = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(_) => return false,
        };

        for (_, drone_id) in p.drones {
            if drone_id == id {
                return true;
            }
        }

        false
    }

    /// Devuelve los drones en el archivo de persistencia
    pub fn get_drones() -> Vec<(Location, u32)> {
        let file_path = Self::get_filepath();
        if std::fs::metadata(file_path.clone()).is_err() {
            return Vec::new();
        }

        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => return Vec::new(),
        };

        let p: Persistence = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        p.drones
    }

    /// Agrega un dron al archivo de persistencia
    pub fn add_drone_to_file(location: Location, id: u32) -> Result<(), DroneError> {
        let file_path = Self::get_filepath();
        if std::fs::metadata(file_path.clone()).is_err() {
            let p = Persistence::new();
            let json = serde_json::to_string(&p).map_err(|_| {
                DroneError::CentralError(
                    "Error while serializing the drone center persistent file".to_string(),
                )
            })?;
            std::fs::write(file_path.clone(), json).map_err(|_| {
                DroneError::CentralError(
                    "Error while writing the drone center persistent file".to_string(),
                )
            })?;
        }

        let file = File::open(file_path.clone()).map_err(|_| {
            DroneError::CentralError(
                "Error while opening the drone center persistent file".to_string(),
            )
        })?;
        let mut p: Persistence = serde_json::from_reader(file).map_err(|_| {
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
        std::fs::write(file_path, json).map_err(|_| {
            DroneError::CentralError(
                "Error while writing the drone center persistent file".to_string(),
            )
        })?;

        Ok(())
    }

    /// Elimina un dron con el id dado del archivo de persistencia
    pub fn remove_drone_from_file(id: u32) -> Result<(), DroneError> {
        let file_path = Self::get_filepath();
        if !Persistence::drone_exists(id) {
            return Ok(());
        }
        let file = File::open(file_path.clone()).map_err(|_| {
            DroneError::CentralError(
                "Error while opening the drone center persistent file".to_string(),
            )
        })?;
        let mut p: Persistence = serde_json::from_reader(file).map_err(|_| {
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
            std::fs::write(file_path, json).map_err(|_| {
                DroneError::CentralError(
                    "Error while writing the drone center persistent file".to_string(),
                )
            })?;
        }
        Ok(())
    }

    /// Devuelve los incidentes en el archivo de persistencia
    pub fn get_incidents() -> Vec<Location> {
        let file_path = Self::get_filepath();
        if std::fs::metadata(file_path.clone()).is_err() {
            return Vec::new();
        }

        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => return Vec::new(),
        };

        let p: Persistence = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        p.incidents
    }

    /// Agrega un incidente al archivo de persistencia
    pub fn add_incident_to_file(location: Location) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = Self::get_filepath();
        if std::fs::metadata(file_path.clone()).is_err() {
            let p = Persistence::new();
            let json = serde_json::to_string(&p)?;
            std::fs::write(file_path.clone(), json)?;
        }

        let file = File::open(file_path.clone())?;

        let mut p: Persistence = serde_json::from_reader(file)?;

        p.incidents.push(location);

        let json = serde_json::to_string(&p)?;
        std::fs::write(file_path, json)?;

        Ok(())
    }

    /// Elimina un incidente con la location dada del archivo de persistencia
    pub fn remove_incident_from_file(location: Location) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = Self::get_filepath();
        if std::fs::metadata(file_path.clone()).is_err() {
            return Ok(());
        }

        let file = File::open(file_path.clone())?;
        let mut p: Persistence = serde_json::from_reader(file)?;

        if let Some(index) = p.incidents.iter().position(|x| *x == location) {
            p.incidents.remove(index);
            let json = serde_json::to_string(&p)?;
            std::fs::write(file_path, json)?;
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

    use super::*;

    #[test]
    fn persistence() {
        assert!(test_add_and_remove_camera_to_file().is_ok());
        assert!(test_add_and_remove_central_to_file().is_ok());
        assert!(test_add_and_remove_drone_to_file().is_ok());
        assert!(test_add_incident_to_file().is_ok());
    }

    fn test_add_and_remove_camera_to_file() -> Result<(), Box<dyn std::error::Error>> {
        
        if std::fs::metadata(TEST_PERSISTENCE_FILE).is_ok() {
            std::fs::remove_file(TEST_PERSISTENCE_FILE).unwrap();
        }
        
        let camera: Camera = Camera::new(Location::new(1.0, 1.0), 0).unwrap();

        let _ = Persistence::add_camera_to_file(camera);

        let cameras = Persistence::get_cameras();
        if cameras.len() != 1 || cameras[0].id != 0 {
            return Err("Error: Camera not added to file".into());
        }

        Persistence::remove_camera_from_file(0).unwrap();

        let cameras = Persistence::get_cameras();
        if cameras.len() != 0 {
            return Err("Error: Camera not removed from file".into());
        }

        std::fs::remove_file(TEST_PERSISTENCE_FILE).unwrap();

        Ok(())
    }

    fn test_add_and_remove_central_to_file() -> Result<(), Box<dyn std::error::Error>> {
        if std::fs::metadata(TEST_PERSISTENCE_FILE).is_ok() {
            std::fs::remove_file(TEST_PERSISTENCE_FILE).unwrap();
        }

        let _ = Persistence::add_center_to_file(
            0,
            Location::new(1.0, 1.0),
            "packets_config/drone_config.json".to_string(),
            "127.0.0.1:5098".to_string(),
        );

        let centrals = Persistence::get_centrals();
        if centrals[0].location != Location::new(1.0, 1.0)
            || centrals[0].id != 0
        {
            return Err("Error: Central not added to file".into());
        }

        Persistence::remove_central_from_file(0).unwrap();

        let centrals = Persistence::get_centrals();
        if centrals.len() != 0 {
            return Err("Error: Central not removed from file".into());
        }

        std::fs::remove_file(TEST_PERSISTENCE_FILE).unwrap();
        Ok(())
    }

    fn test_add_and_remove_drone_to_file() -> Result<(), Box<dyn std::error::Error>> {
        if std::fs::metadata(TEST_PERSISTENCE_FILE).is_ok() {
            std::fs::remove_file(TEST_PERSISTENCE_FILE).unwrap();
        }

        let _ = Persistence::add_drone_to_file(Location::new(1.0, 1.0), 0);

        let drones = Persistence::get_drones();
        if drones.len() != 1 || drones[0].1 != 0 {
            return Err("Error: Drone not added to file".into());
        }

        let _ = Persistence::remove_drone_from_file(0);

        let drones = Persistence::get_drones();
        if drones.len() != 0 {
            return Err("Error: Drone not removed from file".into());
        }

        std::fs::remove_file(TEST_PERSISTENCE_FILE).unwrap();
        Ok(())
    }

    fn test_add_incident_to_file() -> Result<(), Box<dyn std::error::Error>> {
        if std::fs::metadata(TEST_PERSISTENCE_FILE).is_ok() {
            std::fs::remove_file(TEST_PERSISTENCE_FILE).unwrap();
        }

        let _ = Persistence::add_incident_to_file(Location::new(1.0, 1.0));

        let incidents = Persistence::get_incidents();
        if incidents.len() != 1 || incidents[0] != Location::new(1.0, 1.0) {
            return Err("Error: Incident not added to file".into());
        }

        Persistence::remove_incident_from_file(Location::new(1.0, 1.0)).unwrap();

        let incidents = Persistence::get_incidents();
        if incidents.len() != 0 {
            return Err("Error: Incident not removed from file".into());
        }
        std::fs::remove_file(TEST_PERSISTENCE_FILE).unwrap();
        Ok(())
    }
}