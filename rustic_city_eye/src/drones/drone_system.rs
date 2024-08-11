use egui::ahash::HashMap;

use crate::drones::drone_center::DroneCenter;
use crate::monitoring::persistence::Persistence;
use crate::mqtt::protocol_error::ProtocolError;
use crate::utils::location::Location;

use super::drone_error::DroneError;
/// Gestiona centros de drones y drones.
/// Nos permite añadir, recuperar, mover y eliminar drones y centros de drones.
#[derive(Debug)]
pub struct DroneSystem {
    pub drone_centers: HashMap<u32, DroneCenter>,
    drone_config_path: String,
    address: String,
}

impl DroneSystem {
    pub fn new(drone_config_path: String, address: String) -> DroneSystem {
        Self {
            drone_centers: HashMap::default(),
            drone_config_path,
            address,
        }
    }

    pub fn disconnect_system(&mut self) -> Result<(), ProtocolError> {
        for center in self.drone_centers.values_mut() {
            center.disconnect_drones()?;
        }

        Ok(())
    }

    pub fn load_existing_drone_center(&mut self, location: Location) -> Result<u32, DroneError> {
        let mut id = 0;
        while self.drone_centers.contains_key(&id) {
            id += 1;
        }

        let drone_center = DroneCenter::new(
            id,
            location,
            self.drone_config_path.to_string(),
            self.address.to_string(),
        );

        self.drone_centers.insert(id, drone_center);

        println!("Drone center id: {} loaded successfully", id);

        Ok(id)
    }

    /// Agrega un nuevo centro de drones al sistema de drones.
    ///
    /// Devuelve su id o DroneError en caso de error.
    pub fn add_drone_center(&mut self, location: Location) -> Result<u32, DroneError> {
        let mut id = 0;
        while self.drone_centers.contains_key(&id) {
            id += 1;
        }

        let drone_center = DroneCenter::new(
            id,
            location,
            self.drone_config_path.to_string(),
            self.address.to_string(),
        );

        self.drone_centers.insert(id, drone_center);
        let _ = Persistence::add_central_to_json(
            id,
            location,
            self.drone_config_path.to_string(),
            self.address.to_string(),
        );
        println!("Drone center id: {} added successfully", id);

        Ok(id)
    }

    /// Carga un dron existente y lo agrega al centro de drones especificado segun ID
    pub fn load_existing_drone(
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
        let _ = Persistence::add_drone_to_json(location, id);
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Condvar, Mutex},
        thread::{self},
    };

    use crate::{mqtt::broker::Broker, utils::location};

    use super::*;

    #[test]
    fn test_01_drone_system_add_drone_center_ok() {
        let mut drone_system = DroneSystem::new(
            "src/drones/drone_config.json".to_string(),
            "127.0.0.1:5099".to_string(),
        );
        let location = location::Location::new(0.0, 0.0);
        let _ = drone_system.add_drone_center(location);

        assert_eq!(drone_system.drone_centers.len(), 1);
    }

    #[test]
    fn test_02_drone_system_add_drone_ok() {
        let args = vec!["127.0.0.1".to_string(), "5098".to_string()];
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => panic!("Error creating broker: {:?}", e),
        };

        let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
        let server_ready_clone = server_ready.clone();
        thread::spawn(move || {
            {
                let (lock, cvar) = &*server_ready_clone;
                let mut ready = match lock.lock() {
                    Ok(ready) => ready,
                    Err(e) => panic!("Error locking mutex: {:?}", e),
                };
                *ready = true;
                cvar.notify_all();
            }
            let _ = broker.server_run();
        });

        // Wait for the server to start
        {
            let (lock, cvar) = &*server_ready;
            let mut ready = match lock.lock() {
                Ok(ready) => ready,
                Err(e) => panic!("Error locking mutex: {:?}", e),
            };
            while !*ready {
                ready = match cvar.wait(ready) {
                    Ok(ready) => ready,
                    Err(e) => panic!("Error waiting for condition variable: {:?}", e),
                };
            }
        }
        let handle = thread::spawn(move || {
            let mut drone_system = DroneSystem::new(
                "src/drones/drone_config.json".to_string(),
                "127.0.0.1:5098".to_string(),
            );
            let location = location::Location::new(0.0, 0.0);
            let _ = drone_system.add_drone_center(location);

            let location = location::Location::new(0.0, 0.0);
            let id = match drone_system.add_drone(location, 0) {
                Ok(id) => id,
                Err(e) => panic!("Error adding drone: {:?}", e),
            };
            println!("dron id {}", id);
            assert_eq!(drone_system.drone_centers[&id].get_drones().len(), 1);
        });

        handle.join().unwrap();
    }

    #[test]
    fn test_03_drone_system_add_drone_center_not_found() {
        let args = vec!["127.0.0.1".to_string(), "5002".to_string()];
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => panic!("Error creating broker: {:?}", e),
        };

        let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
        let server_ready_clone = server_ready.clone();
        thread::spawn(move || {
            {
                let (lock, cvar) = &*server_ready_clone;
                let mut ready = lock.lock().unwrap();
                *ready = true;
                cvar.notify_all();
            }
            let _ = broker.server_run();
        });

        // Wait for the server to start
        {
            let (lock, cvar) = &*server_ready;
            let mut ready = lock.lock().unwrap();
            while !*ready {
                ready = cvar.wait(ready).unwrap();
            }
        }

        let handle = thread::spawn(move || {
            let mut drone_system = DroneSystem::new(
                "src/drones/drone_config.json".to_string(),
                "127.0.0.1:5002".to_string(),
            );
            let location = location::Location::new(0.0, 0.0);
            let _ = drone_system.add_drone_center(location);

            let location = location::Location::new(0.0, 0.0);
            match drone_system.add_drone(location, 1) {
                Ok(_) => (),
                Err(e) => assert_eq!(e, DroneError::DroneCenterNotFound),
            };
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_04_connect_drone() {
        let args = vec!["127.0.0.1".to_string(), "5097".to_string()];
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => panic!("Error creating broker: {:?}", e),
        };

        let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
        let server_ready_clone = server_ready.clone();
        thread::spawn(move || {
            {
                let (lock, cvar) = &*server_ready_clone;
                let mut ready = lock.lock().unwrap();
                *ready = true;
                cvar.notify_all();
            }
            let _ = broker.server_run();
        });

        // Wait for the server to start
        {
            let (lock, cvar) = &*server_ready;
            let mut ready = lock.lock().unwrap();
            while !*ready {
                ready = cvar.wait(ready).unwrap();
            }
        }
        let t1 = thread::spawn(move || {
            let mut drone_system = DroneSystem::new(
                "src/drones/drone_config.json".to_string(),
                "127.0.0.1:5097".to_string(),
            );
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
