#[cfg(test)]
mod tests {
    
    use std::{
        path::PathBuf, sync::{mpsc, Arc, Condvar, Mutex}, thread
    };

    use broker::broker::Broker;
    use chrono::Utc;
    use utils::location::Location;

    use crate::{drone::Drone, drone_error::DroneError, drone_state::DroneState};

    #[test]
    fn test_01_reading_connect_config_ok() -> Result<(), DroneError> {
        let project_dir = env!("CARGO_MANIFEST_DIR");
        let file_path = PathBuf::from(project_dir).join("packets_config/connect_config.json");
        let id = 1;
        let connect_config = Drone::read_connect_config(file_path.to_str().unwrap(), id);

        match connect_config {
            Ok(config) => {
                println!("Connect config: {:?}", config);

                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    #[test]
    fn test_02_drone_arranca_en_waiting_state() {
        let args = vec!["127.0.0.1".to_string(), "5001".to_string()];
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

        let (_disconnect_sender, disconnect_receiver) = mpsc::channel();
  
        let handle = thread::spawn(move || {
            let latitude = 0.0;
            let longitude = 0.0;
            let location = Location::new(latitude, longitude);
            let center_location = Location::new(0.0, 0.0);
            let mut drone = Drone::new(
                1,
                location,
                center_location,
                "packets_config/drone_config.json",
                "127.0.0.1:5001".to_string(),
                disconnect_receiver,
            ).unwrap();

            let _ = drone.run_drone();

            assert_eq!(drone.get_state(), DroneState::Waiting);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_03_get_id_ok() {
        let args = vec!["127.0.0.1".to_string(), "5003".to_string()];
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

        let (_disconnect_sender, disconnect_receiver) = mpsc::channel();

        let handle = thread::spawn(move || {
            let latitude = 0.0;
            let longitude = 0.0;
            let location = Location::new(latitude, longitude);
            let center_location = Location::new(0.0, 0.0);
            let drone = Drone::new(
                1,
                location,
                center_location,
                "packets_config/drone_config.json",
                "127.0.0.1:5003".to_string(),
                disconnect_receiver,
            )
            .unwrap();

            assert_eq!(drone.get_id(), 1);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_04_drone_bad_config_file() {
        let latitude = 0.0;
        let longitude = 0.0;
        let (_disconnect_sender, disconnect_receiver) = mpsc::channel();

        let location = Location::new(latitude, longitude);
        let center_location = Location::new(0.0, 0.0);
        let drone = Drone::new(
            1,
            location,
            center_location,
            "./tests/bad_config_file.json",
            "127.0.0.1:5000".to_string(),
            disconnect_receiver,
        );
        assert!(drone.is_err());
    }

    #[test]
    fn test_05_drone_stays_within_radius() {
        let args = vec!["127.0.0.1".to_string(), "5004".to_string()];
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
            let latitude = 0.0;
            let longitude = 0.0;
            let (_disconnect_sender, disconnect_receiver) = mpsc::channel();

            let location = Location::new(latitude, longitude);
            let center_location = Location::new(0.0, 0.0);
            let drone = Drone::new(
                1,
                location,
                center_location,
                "packets_config/drone_config.json",
                "127.0.0.1:5004".to_string(),
                disconnect_receiver,
            )
            .unwrap();
            let radius = 0.005;
            // let _ = drone.drone_movement(target_location);

            let distance_from_center =
                ((drone.location.lat).powi(2) + (drone.location.long).powi(2)).sqrt();
            assert!(
                distance_from_center <= radius,
                "Drone should stay within the radius."
            );
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_06_new_drone() {
        let args = vec!["127.0.0.1".to_string(), "5007".to_string()];
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
            let location = Location {
                lat: 0.0,
                long: 0.0,
            };
            let center_location = Location {
                lat: 0.0,
                long: 0.0,
            };
            let config_file_path = "packets_config/drone_config.json";
            let address = "127.0.0.1:5007".to_string();
            let (_disconnect_sender, disconnect_receiver) = mpsc::channel();

            let drone: Result<Drone, DroneError> = Drone::new(
                1,
                location,
                center_location,
                config_file_path,
                address,
                disconnect_receiver,
            );

            assert!(drone.is_ok());
            let drone = drone.unwrap();
            assert_eq!(drone.id, 1);
            assert_eq!(drone.location, location);
            assert_eq!(drone.center_location, center_location);
            assert_eq!(drone.battery_level, 100);
            assert_eq!(drone.drone_state, DroneState::Waiting);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_07_battery_discharge() {
        let args = vec!["127.0.0.1".to_string(), "5008".to_string()];
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
        let mut drone = setup_test_drone("127.0.0.1:5008".to_string());
        for _ in 0..100 {
            drone.update_battery_discharge(Utc::now(), Utc::now() + chrono::Duration::seconds(120));
        }

        assert!(drone.battery_level <= 0);
        assert_eq!(drone.drone_state, DroneState::LowBatteryLevel);
    }

    #[test]
    fn test_08_charge_battery() {
        let args = vec!["127.0.0.1".to_string(), "5009".to_string()];
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

        let mut drone = setup_test_drone("127.0.0.1:5009".to_string());
        for _ in 0..100 {
            drone.update_battery_discharge(Utc::now(), Utc::now() + chrono::Duration::seconds(120));
        }

        assert!(drone.battery_level <= 0);
        assert_eq!(drone.drone_state, DroneState::LowBatteryLevel);

        for _ in 0..100 {
            drone.update_battery_charge(Utc::now(), Utc::now() + chrono::Duration::seconds(120));
        }
        assert_eq!(drone.battery_level, 100);
        assert_eq!(drone.get_state(), DroneState::Waiting);
    }

    #[test]
    fn test_09_drone_movement() {
        let args = vec!["127.0.0.1".to_string(), "5010".to_string()];
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
            let mut drone = setup_test_drone("127.0.0.1:5010".to_string());
            drone.location = Location {
                lat: 0.0,
                long: 0.0,
            };
            let drone_arc = Arc::new(Mutex::new(drone));
            for _ in 0..14 {
                let mut drone = drone_arc.lock().unwrap();
                drone.calculate_new_position();
            }
            let drone = drone_arc.lock().unwrap();
            let new_location = &drone.location;
            println!("New location: {:?}", new_location);
            assert!(new_location.lat > -1.0 && new_location.lat < 1.0);
            assert!(new_location.long > -1.0 && new_location.long < 1.0);
        });
        handle.join().unwrap();
    }

    // Helper function to setup a test drone
    fn setup_test_drone(addres: String) -> Drone {
        let location = Location {
            lat: 0.0,
            long: 0.0,
        };
        let center_location = Location {
            lat: 0.0,
            long: 0.0,
        };
        let config_file_path = "packets_config/drone_config.json";
        let address = addres.to_string();
        let (_disconnect_sender, disconnect_receiver) = mpsc::channel();

        Drone::new(
            1,
            location,
            center_location,
            config_file_path,
            address,
            disconnect_receiver,
        )
        .unwrap()
    }
}