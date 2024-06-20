use egui::ahash::HashMap;
use rand::Rng;

use crate::drones::drone_center::DroneCenter;
use crate::utils::location::Location;

use super::drone_error::DroneError;

#[derive(Debug)]
pub struct DroneSystem {
    drone_centers: HashMap<u32, DroneCenter>,
    drone_config_path: String,
    address: String,
}

/// The `DroneSystem` struct represents a system that manages drone centers and drones.
/// It provides methods to add, retrieve, move, and remove drones and drone centers.
impl DroneSystem {
    /// Creates a new instance of `DroneSystem`.
    pub fn new(drone_config_path: String, address: String) -> DroneSystem {
        Self {
            drone_centers: HashMap::default(),
            drone_config_path,
            address,
        }
    }

    /// Agrega un nuevo centro de drones al sistema de drones.
    ///
    /// Devuelve su id o DroneError en caso de error.
    pub fn add_drone_center(&mut self, location: Location) -> Result<u32, DroneError> {
        // let mut rng = rand::thread_rng();

        // let mut id = rng.gen();

        let mut id = 0;
        while self.drone_centers.contains_key(&id) {
            id = id + 1;
        }

        let drone_center = DroneCenter::new(
            id,
            location,
            self.drone_config_path.to_string(),
            self.address.to_string(),
        );
        self.drone_centers.insert(id, drone_center);
        Ok(id)
    }

    /// Agrega un nuevo dron al centro de drones especificado segun ID
    ///
    /// Devuelve el id del drone o DroneError en caso de error.
    pub fn add_drone(
        &mut self,
        location: Location,
        drone_center_id: u32,
    ) -> Result<u32, DroneError> {
        let drone_center = match self.drone_centers.get_mut(&drone_center_id) {
            Some(drone_center) => drone_center,
            None => return Err(DroneError::DroneCenterNotFound),
        };

        let id = drone_center.add_drone(location)?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use crate::{mqtt::broker::Broker, utils::location};

    use super::*;

    #[test]
    fn test_01_drone_system_add_drone_center_ok() {
        let mut drone_system = DroneSystem::new("".to_string(), "".to_string());
        let location = location::Location::new(0.0, 0.0);
        let _ = drone_system.add_drone_center(location);

        assert_eq!(drone_system.drone_centers.len(), 1);
    }

    #[test]
    fn test_02_drone_system_add_drone_ok() {
        let args = vec!["127.0.0.1".to_string(), "5000".to_string()];
        let addr = "127.0.0.1:5000";
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => {
                panic!("Error creating broker: {:?}", e)
            }
        };
        thread::spawn(move || {
            let _ = broker.server_run();
        });
        thread::spawn(move || {
            let mut drone_system = DroneSystem::new(
                "src/drone_system/drone_config.json".to_string(),
                addr.to_string(),
            );
            let location = location::Location::new(0.0, 0.0);
            let _ = drone_system.add_drone_center(location);

            let location = location::Location::new(0.0, 0.0);
            let id = match drone_system.add_drone(location, 0) {
                Ok(id) => id,
                Err(e) => panic!("Error adding drone: {:?}", e),
            };

            assert_eq!(drone_system.drone_centers[&id].get_drones().len(), 1);
        });
    }

    #[test]
    fn test_03_drone_system_add_drone_center_not_found() {
        let args = vec!["127.0.0.1".to_string(), "5000".to_string()];
        let addr = "127.0.0.1:5000";
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => {
                panic!("Error creating broker: {:?}", e)
            }
        };
        thread::spawn(move || {
            let _ = broker.server_run();
        });

        thread::spawn(move || {
            let mut drone_system = DroneSystem::new(
                "src/drone_system/drone_config.json".to_string(),
                addr.to_string(),
            );
            let location = location::Location::new(0.0, 0.0);
            let _ = drone_system.add_drone_center(location);

            let location = location::Location::new(0.0, 0.0);
            match drone_system.add_drone(location, 1) {
                Ok(_) => (),
                Err(e) => assert_eq!(e, DroneError::DroneCenterNotFound),
            };
        });
    }

    #[test]
    fn test_04_connect_drone() {
        let args = vec!["127.0.0.1".to_string(), "5000".to_string()];
        let addr = "127.0.0.1:5000";
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => {
                panic!("Error creating broker: {:?}", e)
            }
        };
        thread::spawn(move || {
            let _ = broker.server_run();
        });

        let t1 = thread::spawn(move || {
            let mut drone_system =
                DroneSystem::new("src/drones/drone_config.json".to_string(), addr.to_string());
            let location = location::Location::new(0.0, 0.0);
            let id = drone_system.add_drone_center(location);

            let location = location::Location::new(0.0, 0.0);
            match drone_system.add_drone(location, id.unwrap()) {
                Ok(_) => (),
                Err(e) => panic!("Error adding drone: {:?}", e),
            };
        });
        t1.join().unwrap();
    }
}
