use std::sync::mpsc::{self, Sender};

use egui::ahash::HashMap;
use rand::Rng;

use crate::drones::drone::Drone;
use crate::mqtt::protocol_error::ProtocolError;
use crate::utils::location::Location;

use super::drone_error::DroneError;
#[derive(Debug)]

///Un drone cententer esta compuesto por sus diferentes drones.
/// Tiene su ID unico, su ubicación, la dirección a la que se conecta
/// y el path de su configuración.
pub struct DroneCenter {
    pub id: u32,
    pub location: Location,
    pub drones: HashMap<u32, Drone>,
    drone_config_path: String,
    address: String,

    disconnect_senders: Vec<Sender<()>>
}

impl DroneCenter {
    pub fn new(
        id: u32,
        location: Location,
        drone_config_path: String,
        address: String,
    ) -> DroneCenter {
        Self {
            id,
            location,
            drones: HashMap::default(),
            drone_config_path,
            address,
            disconnect_senders: Vec::new()
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_location(&self) -> Location {
        self.location
    }

    pub fn disconnect_drones(&mut self) -> Result<(), ProtocolError> {
        for drone in self.drones.values_mut() {
            drone.disconnect()?;
        }

        for sender in self.disconnect_senders.clone() {
            sender.send(()).unwrap();
        }

        Ok(())
    }

    /// Crea un dron y lo agrega al hashmap con un ID que no esté siendo utilizado
    ///
    /// Retorna el ID del dron creado o DroneError en caso de error.
    pub fn add_drone(&mut self, location: Location) -> Result<u32, DroneError> {
        let mut id = 0;
        while self.drones.contains_key(&id) {
            id += 1;
        }

        let (disconnect_sender, disconnect_receiver) = mpsc::channel();

        let mut drone = Drone::new(
            id,
            location,
            self.location,
            &self.drone_config_path.to_string(),
            self.address.to_string(),
            disconnect_receiver
        )?;

        self.disconnect_senders.push(disconnect_sender);

        drone.run_drone()?;
        self.drones.insert(id, drone);
        Ok(id)
    }

    pub fn get_drones(&self) -> &HashMap<u32, Drone> {
        &self.drones
    }

    /// Retorna un dron aleatorio del centro de drones
    pub fn get_drone(&self) -> Option<&Drone> {
        let keys: Vec<&u32> = self.drones.keys().collect();
        if keys.is_empty() {
            None
        } else {
            let idx = rand::thread_rng().gen_range(0..keys.len());
            self.drones.get(keys[idx])
        }
    }

    /// Retorna un dron del centro de drones segun su ID
    pub fn get_drone_by_id(&self, id: u32) -> Option<&Drone> {
        let drone = self.drones.get(&id);

        drone
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::thread;

    use crate::{mqtt::broker::Broker, utils::location};

    use super::*;

    #[test]
    fn test_01_drone_center_add_drone_ok() {
        let latitude = 0.0;
        let longitude = 0.0;
        let location = location::Location::new(latitude, longitude);
        // Set up a listener on a local port.
        let args = vec!["127.0.0.1".to_string(), "5000".to_string()];
        let addr = "127.0.0.1:5000";
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => {
                panic!("Error creating broker: {:?}", e)
            }
        };
        thread::spawn(move || {
            _ = broker.server_run();
        });

        thread::spawn(move || {
            let mut drone_center = DroneCenter::new(
                1,
                location,
                "src/drones/drone_config.json".to_string(),
                addr.to_string(),
            );

            match drone_center.add_drone(location) {
                Ok(_) => {}
                Err(e) => {
                    panic!("Error adding drone to drone center: {:?}", e)
                }
            };

            let drones = drone_center.get_drones();

            assert_eq!(drones.len(), 1);
        });
    }

    #[test]
    fn test_02_drone_center_get_drone_ok() {
        let args = vec!["127.0.0.1".to_string(), "5000".to_string()];
        let addr = "127.0.0.1:5000";
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => {
                panic!("Error creating broker: {:?}", e)
            }
        };
        thread::spawn(move || {
            _ = broker.server_run();
        });

        thread::spawn(move || {
            let latitude = 0.0;
            let longitude = 0.0;
            let location = location::Location::new(latitude, longitude);
            let mut drone_center = DroneCenter::new(
                1,
                location,
                "./src/drones/drone_config.json".to_string(),
                addr.to_string(),
            );

            let id = match drone_center.add_drone(location) {
                Ok(id) => id,
                Err(e) => {
                    panic!("Error adding drone to drone center: {:?}", e)
                }
            };

            let drone = drone_center.get_drone_by_id(id);

            assert!(drone.is_some());
        });
    }

    #[test]
    fn test_03_drone_center_get_drone_none() {
        let args = vec!["127.0.0.1".to_string(), "5000".to_string()];
        let addr = "127.0.0.1:5000";
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => {
                panic!("Error creating broker: {:?}", e)
            }
        };
        thread::spawn(move || {
            _ = broker.server_run();
        });

        thread::spawn(move || {
            let latitude = 0.0;
            let longitude = 0.0;
            let location = location::Location::new(latitude, longitude);
            let mut drone_center = DroneCenter::new(
                1,
                location,
                "./src/drones/drone_config.json".to_string(),
                addr.to_string(),
            );

            let id = drone_center.add_drone(location).unwrap();

            let drone = drone_center.get_drone_by_id(id);

            assert!(drone.is_none());
        });
    }

    #[test]
    fn test_04_drone_center_get_location_ok() {
        let args = vec!["127.0.0.1".to_string(), "5000".to_string()];
        let addr = "127.0.0.1:5000";
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => {
                panic!("Error creating broker: {:?}", e)
            }
        };
        thread::spawn(move || {
            _ = broker.server_run();
        });

        thread::spawn(move || {
            let latitude = 0.0;
            let longitude = 0.0;
            let location = location::Location::new(latitude, longitude);
            let drone_center = DroneCenter::new(
                1,
                location,
                "./src/drones/drone_config.json".to_string(),
                addr.to_string(),
            );

            let drone_center_location = drone_center.get_location();

            assert_eq!(drone_center_location, location);
        });
    }

    #[test]
    fn test_05_get_drones_ok() {
        let args = vec!["127.0.0.1".to_string(), "5000".to_string()];
        let addr = "127.0.0.1:5000";
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => {
                panic!("Error creating broker: {:?}", e)
            }
        };
        thread::spawn(move || {
            _ = broker.server_run();
        });

        thread::spawn(move || {
            let latitude = 0.0;
            let longitude = 0.0;
            let location = location::Location::new(latitude, longitude);
            let mut drone_center = DroneCenter::new(
                1,
                location,
                "./src/drones/drone_config.json".to_string(),
                addr.to_string(),
            );

            let _ = drone_center.add_drone(location);

            let drones = drone_center.get_drones();

            assert_eq!(drones.len(), 1);
        });
    }
}
