use std::sync::{mpsc, Arc, Mutex};

use chrono::Utc;

use super::{drone_config::DroneConfig, drone_error::DroneError, drone_state::DroneState};
use crate::{
    mqtt::{
        client::Client,
        client_message::{self, ClientMessage},
        messages_config,
        publish::publish_config::PublishConfig,
    },
    utils::{location::Location, payload_types::PayloadTypes},
};
#[derive(Debug, Clone)]
pub struct Drone {
    // ID unico para cada Drone.
    id: u32,
    ///posicion actual del Drone.
    location: Location,

    ///posicion del centro de drones al que pertenece.
    center_location: Location,
    /// La configuracion del Drone contiene el nivel de bateria del mismo y
    /// el radio de operacion.
    drone_config: DroneConfig,

    ///  El Drone puede tener distintos estados:
    /// - Waiting: esta circulando en su radio de operacion, pero no esta atendiendo ningun incidente.
    /// - AttendingIncident: un nuevo incidente fue cargado por la app de monitoreo, y el Drone fue asignado
    ///                         a resolverlo.
    /// - LowBatteryLevel: el Drone se quedo sin bateria, por lo que va a su central a cargarse, y no va a volver a
    ///                    funcionar hasta que tenga el nivel de bateria completo(al terminar de cargarse, vuelve a
    ///                    tener el estado Waiting).
    drone_state: DroneState,

    drone_client: Client,

    battery_level: i64,

    send_to_client_channel: mpsc::Sender<Box<dyn messages_config::MessagesConfig + Send>>,

    #[allow(dead_code)]
    recieve_from_client: Arc<Mutex<mpsc::Receiver<ClientMessage>>>,
}

impl Drone {
    /// levanto su configuracion, y me guardo su posicion inicial.
    pub fn new(
        id: u32,
        location: Location,
        center_location: Location,
        config_file_path: &str,
        address: String,
    ) -> Result<Drone, DroneError> {
        let drone_config = DroneConfig::new(config_file_path)?;

        let connect_config =
            match client_message::Connect::read_connect_config("src/drones/connect_config.json") {
                Ok(config) => config,
                Err(e) => return Err(DroneError::ProtocolError(e.to_string())),
            };
        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        let drone_client = match Client::new(rx, address, connect_config, tx2) {
            Ok(client) => client,
            Err(e) => return Err(DroneError::ProtocolError(e.to_string())),
        };
        Ok(Drone {
            id,
            location,
            center_location,
            drone_config,
            drone_state: DroneState::Waiting,
            drone_client,
            battery_level: 100,
            send_to_client_channel: tx,
            recieve_from_client: Arc::new(Mutex::new(rx2)),
        })
    }

    /// Esta funcion ejecuta el dron, creando dos hilos:
    /// Uno para la descarga de bateria y otro para el movimiento del dron.
    /// Cuando la bateria se descarga por completo dentro del thread de descarga de bateria,
    /// se cambia el estado del dron a LowBatteryLevel. Y, en el thread de movimiento, se
    /// redirije al dron hacia su estacion de carga.
    pub fn run_drone(&mut self) -> Result<(), DroneError> {
        match self.drone_client.client_run() {
            Ok(client) => client,
            Err(e) => return Err(DroneError::ProtocolError(e.to_string())),
        };

        let self_clone = Arc::new(Mutex::new(self.clone()));
        std::thread::spawn(move || {
            let self_clone: Arc<Mutex<Drone>> = Arc::clone(&self_clone);
            let mut self_locked = self_clone.lock().unwrap();
            let _ = self_locked.battery_discharge();
        });
        let self_clone2 = Arc::new(Mutex::new(self.clone()));

        std::thread::spawn(move || {
            let self_clone: Arc<Mutex<Drone>> = Arc::clone(&self_clone2);
            let mut self_locked = self_clone.lock().unwrap();
            let location_clone = self_locked.location.clone();
            let operation_radius = self_locked.drone_config.get_operation_radius();
            let movement_rate = self_locked.drone_config.get_movement_rate();
            let target_location: Location = self_locked.center_location.clone(); // TODO: placeHOLDER

            self_locked.drone_movement(
                location_clone,
                operation_radius,
                movement_rate,
                target_location,
            )
        });

        Ok(())
    }

    pub fn get_state(self) -> DroneState {
        self.drone_state
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    /// closure del thread de descarga de bateria del Drone.
    ///
    /// Lo que se hace es ir descargando el nivel de bateria del
    /// Drone segun indique la tasa de descarga de bateria del mismo(definida
    /// en la config del Drone).
    ///
    /// Cada vez que se cumpla "un ciclo" de la tasa de descarga, se reduce la bateria del
    /// Drone en un 1%.
    pub fn battery_discharge(&mut self) -> Result<(), DroneError> {
        let mut last_discharge_time = Utc::now();
        loop {
            let current_time = Utc::now();
            let elapsed_time = current_time
                .signed_duration_since(last_discharge_time)
                .num_milliseconds();

            if elapsed_time >= self.drone_config.get_battery_discharge_rate() {
                self.battery_level -= 1;

                if self.battery_level < 0 {
                    self.battery_level = 0;
                    self.drone_state = DroneState::LowBatteryLevel;
                    break;
                }
                last_discharge_time = current_time;
            }
        }
        Ok(())
    }

    /// Carga al Drone de acuerdo a la tasa de carga que venga definida en la configuracion.
    ///
    /// Al llegar al 100%, devuelve un DroneState del tipo Waiting.
    pub fn charge_battery(&mut self) -> Result<DroneState, DroneError> {
        let mut start_time = Utc::now();

        loop {
            let current_time = Utc::now();
            let elapsed_time = current_time
                .signed_duration_since(start_time)
                .num_milliseconds();

            if elapsed_time >= self.drone_config.get_battery_charge_rate_milisecs() {
                self.battery_level += 1;

                if self.battery_level > 100 {
                    self.battery_level = 100;
                    return Ok(DroneState::Waiting);
                }
                start_time = current_time;
            }
        }
    }

    /// closure del thread de movimiento del Drone.
    ///
    /// Lo que se hace es generar una direccion de movimiento(utilizando rand),
    /// y se intenta generar la nueva posicion, siempre respetando el area de operacion
    /// del Drone (center_lat).
    ///
    /// Si se generase una posicion que esta por fuera del area, se van a
    /// generar las direcciones aleatorias que sean necesarias hasta obtener una que nos
    /// lleve de nuevo adentro del area.
    ///
    /// Se utiliza la tasa de movimiento del Drone, que viene definida en la configuracion:
    /// la idea es que el Drone se mueva cada cierto intervalo de tiempo definido por esta tasa de movimiento.
    ///
    fn drone_movement(
        &mut self,
        center_location: Location,
        radius: f64,
        movement_rate: i64,
        target_location: Location,
    ) -> Result<(), DroneError> {
        let center_lat = center_location.lat;
        let center_long = center_location.long;
        let target_lat = target_location.lat;
        let target_long = target_location.long;
        let mut current_lat = self.location.lat;
        let mut current_long = self.location.long;
        let mut last_move_time = Utc::now();

        loop {
            if self.battery_level > 0 {
                let current_time = Utc::now();
                let elapsed_time = current_time
                    .signed_duration_since(last_move_time)
                    .num_milliseconds();

                if elapsed_time >= movement_rate {
                    // Calculate the direction vector towards the target
                    let direction_lat = target_lat - current_lat;
                    let direction_long = target_long - current_long;

                    // Normalize the direction vector
                    let magnitude = (direction_lat.powi(2) + direction_long.powi(2)).sqrt();
                    let unit_direction_lat = direction_lat / magnitude;
                    let unit_direction_long = direction_long / magnitude;

                    // Define the movement step size
                    let step_size = 0.1; // Adjust the step size as needed

                    // Calculate the new position
                    let mut new_lat = current_lat + unit_direction_lat * step_size;
                    let mut new_long = current_long + unit_direction_long * step_size;

                    // Check if the new position is within the radius
                    let distance_from_center =
                        ((new_lat - center_lat).powi(2) + (new_long - center_long).powi(2)).sqrt();
                    if distance_from_center > radius {
                        // If out of bounds, set the new position to the edge of the radius in the direction of the target
                        let scaling_factor = radius / distance_from_center;
                        new_lat = center_lat + (new_lat - center_lat) * scaling_factor;
                        new_long = center_long + (new_long - center_long) * scaling_factor;
                    }

                    current_lat = new_lat;
                    current_long = new_long;

                    let location = Location::new(new_lat, new_long);

                    self.location = location;
                    let payload = PayloadTypes::LocationPayload(self.location.clone());

                    let publish_config =
                        match PublishConfig::read_config("src/drones/publish_config.json", payload)
                        {
                            Ok(config) => config,
                            Err(e) => return Err(DroneError::ProtocolError(e.to_string())),
                        };
                    let _ = self.send_to_client_channel.send(Box::new(publish_config));

                    if self.drone_state == DroneState::LowBatteryLevel {
                        //en el caso de tener poca bateria, se guarda su ubi actual, va a cagarse y vuelve
                        let former_location = self.location.clone();
                        match self.drone_movement(
                            self.location.clone(),
                            radius,
                            movement_rate,
                            self.center_location.clone(),
                        ) {
                            Ok(_) => (),
                            Err(e) => {
                                println!("Error moving drone: {:?}", e);
                            }
                        }
                        match self.charge_battery() {
                            Ok(state) => {
                                self.drone_state = state;
                                match self.drone_movement(
                                    self.location.clone(),
                                    radius,
                                    movement_rate,
                                    former_location,
                                ) {
                                    Ok(_) => (),
                                    Err(e) => {
                                        println!("Error moving drone: {:?}", e);
                                    }
                                };
                            }
                            Err(e) => {
                                println!("Error charging drone: {:?}", e);
                            }
                        }
                    }

                    last_move_time = current_time;
                    if (current_lat - target_lat).abs() < 0.1
                        && (current_long - target_long).abs() < 0.1
                    {
                        return Ok(()); // The drone has reached the target location
                    }
                }
            } else {
                self.drone_state = DroneState::LowBatteryLevel;
                return Err(DroneError::BatteryEmpty);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    #[cfg(test)]
    use std::{
        sync::{Arc, Condvar, Mutex},
        thread,
    };

    use crate::{mqtt::broker::Broker, utils::location};

    use super::*;

    #[test]
    fn test_01_drone_low_battery_level_state_ok() {
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

        let handle = thread::spawn(move || {
            let latitude = 0.0;
            let longitude = 0.0;
            let location = location::Location::new(latitude, longitude);
            let center_location = location::Location::new(0.0, 0.0);
            let mut drone = Drone::new(
                1,
                location,
                center_location,
                "./src/drones/drone_config.json",
                "127.0.0.1:5001".to_string(),
            )
            .unwrap();

            let _ = drone.run_drone();

            assert_eq!(drone.get_state(), DroneState::Waiting);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_02_drone_going_to_charge_battery_ok() {
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
            let latitude = 0.0;
            let longitude = 0.0;
            let location = location::Location::new(latitude, longitude);
            let center_location = location::Location::new(0.0, 0.0);
            let mut drone = Drone::new(
                1,
                location,
                center_location,
                "./tests/drone_config_test.json",
                "127.0.0.1:5002".to_string(),
            )
            .unwrap();

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

        let handle = thread::spawn(move || {
            let latitude = 0.0;
            let longitude = 0.0;
            let location = location::Location::new(latitude, longitude);
            let center_location = location::Location::new(0.0, 0.0);
            let drone = Drone::new(
                1,
                location,
                center_location,
                "./tests/drone_config_test.json",
                "127.0.0.1:5003".to_string(),
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
        let location = location::Location::new(latitude, longitude);
        let center_location = location::Location::new(0.0, 0.0);
        let drone = Drone::new(
            1,
            location,
            center_location,
            "./tests/bad_config_file.json",
            "127.0.0.1:5000".to_string(),
        );
        assert!(drone.is_err());
    }

    #[test]
    fn test_drone_stays_within_radius() {
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
            let location = location::Location::new(latitude, longitude);
            let center_location = location::Location::new(0.0, 0.0);
            let mut drone = Drone::new(
                1,
                location,
                center_location,
                "./src/drones/drone_config.json",
                "127.0.0.1:5004".to_string(),
            )
            .unwrap();
            let target_location = location::Location::new(0.001, 0.001);
            let radius = 0.005;
            let _ = drone.drone_movement(drone.location.clone(), radius, 100, target_location);

            let distance_from_center =
                ((drone.location.lat).powi(2) + (drone.location.long).powi(2)).sqrt();
            assert!(
                distance_from_center <= radius,
                "Drone should stay within the radius."
            );
        });
        handle.join().unwrap();
    }

    // #[test]
    // fn test_drone_stops_when_battery_empty() {
    //     let args = vec!["127.0.0.1".to_string(), "5005".to_string()];
    //     let mut broker = match Broker::new(args) {
    //         Ok(broker) => broker,
    //         Err(e) => panic!("Error creating broker: {:?}", e),
    //     };

    //     let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
    //     let server_ready_clone = server_ready.clone();
    //     thread::spawn(move || {
    //         {
    //             let (lock, cvar) = &*server_ready_clone;
    //             let mut ready = lock.lock().unwrap();
    //             *ready = true;
    //             cvar.notify_all();
    //         }
    //         let _ = broker.server_run();
    //     });

    //     // Wait for the server to start
    //     {
    //         let (lock, cvar) = &*server_ready;
    //         let mut ready = lock.lock().unwrap();
    //         while !*ready {
    //             ready = cvar.wait(ready).unwrap();
    //         }
    //     }

    //     let handle = thread::spawn(move || {
    //         let latitude = 0.0;
    //         let longitude = 0.0;
    //         let location = Location::new(latitude, longitude);
    //         let center_location = Location::new(0.0, 0.0);

    //         let drone = Drone::new(
    //             1,
    //             location,
    //             center_location,
    //             "./src/drones/drone_config.json",
    //             "127.0.0.1:5005".to_string(),
    //         )
    //         .unwrap();

    //         let drone = Arc::new(Mutex::new(drone));

    //         let target_location = Location::new(0.001, 0.001);
    //         let drone_clone = drone.clone();
    //         let drone_clone2 = drone.clone();
    //         thread::spawn(move || {
    //             thread::sleep(Duration::from_millis(50));
    //             let mut drone = drone_clone.lock().unwrap();
    //             drone.battery_level = 0;
    //         });

    //         {
    //             let mut drone = drone.lock().unwrap();
    //             let drone_location = drone.location.clone();
    //             let _ = drone.drone_movement(drone_location.clone(), 0.005, 100, target_location);
    //         }
    //         let drone = drone_clone2.lock().unwrap();

    //         assert!(
    //             drone.battery_level == 0,
    //             "Drone should stop when battery is empty."
    //         );
    //     });
    //     handle.join().unwrap();
    // }

    #[test]

    fn test_drone_charge_battery() {
        let args = vec!["127.0.0.1".to_string(), "5006".to_string()];
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
            let location = Location::new(latitude, longitude);
            let center_location = Location::new(0.0, 0.0);

            let drone = Drone::new(
                1,
                location,
                center_location,
                "./src/drones/drone_config.json",
                "127.0.0.1:5006".to_string(),
            )
            .unwrap();

            let drone = Arc::new(Mutex::new(drone));
            let drone_clone = drone.clone();
            thread::spawn(move || {
                let mut drone = drone_clone.lock().unwrap();
                let _ = drone.charge_battery();
            });

            let drone = drone.lock().unwrap();
            assert_eq!(drone.battery_level, 100);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_new_drone() {
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
            let config_file_path = "./src/drones/drone_config.json";
            let address = "127.0.0.1:5007".to_string();

            let drone: Result<Drone, DroneError> = Drone::new(
                1,
                location.clone(),
                center_location.clone(),
                config_file_path,
                address,
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
    fn test_battery_discharge() {
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
        let handle = thread::spawn(move || {
            let mut drone = setup_test_drone("127.0.0.1:5008".to_string());
            drone.battery_level = 10; // Inicializamos con 10% de batería para la prueba

            let _ = drone.battery_discharge();
            assert!(drone.battery_level <= 0);
            assert_eq!(drone.drone_state, DroneState::LowBatteryLevel);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_charge_battery() {
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
        let handle = thread::spawn(move || {
            let mut drone = setup_test_drone("127.0.0.1:5009".to_string());
            drone.battery_level = 0;

            let result = drone.charge_battery();
            assert!(result.is_ok());
            assert_eq!(drone.battery_level, 100);
            assert_eq!(result.unwrap(), DroneState::Waiting);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_drone_movement() {
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
            let target_location = Location {
                lat: 1.0,
                long: 1.0,
            };
            match drone.drone_movement(
                drone.location.clone(),
                drone.drone_config.get_operation_radius(),
                drone.drone_config.get_movement_rate(),
                target_location,
            ) {
                Ok(_) => (),
                Err(e) => {
                    panic!("Error moving drone: {:?}", e);
                }
            };

            let new_location = drone.location;
            assert!(new_location.lat == 1.0 || new_location.long != 1.0);
        });
        handle.join().unwrap();
    }

    // #[test]
    // fn test_drone_movement_out_of_bounds() {
    //     let args = vec!["127.0.0.1".to_string(), "5011".to_string()];
    //     let mut broker = match Broker::new(args) {
    //         Ok(broker) => broker,
    //         Err(e) => panic!("Error creating broker: {:?}", e),
    //     };

    //     let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
    //     let server_ready_clone = server_ready.clone();
    //     thread::spawn(move || {
    //         {
    //             let (lock, cvar) = &*server_ready_clone;
    //             let mut ready = lock.lock().unwrap();
    //             *ready = true;
    //             cvar.notify_all();
    //         }
    //         let _ = broker.server_run();
    //     });

    //     // Wait for the server to start
    //     {
    //         let (lock, cvar) = &*server_ready;
    //         let mut ready = lock.lock().unwrap();
    //         while !*ready {
    //             ready = cvar.wait(ready).unwrap();
    //         }
    //     }
    //     let handle = thread::spawn(move || {
    //         let mut drone = setup_test_drone("127.0.0.1:5011".to_string());
    //         drone.location = Location {
    //             lat: 0.0,
    //             long: 0.0,
    //         };
    //         let target_location = Location {
    //             lat: 1.0,
    //             long: 1.0,
    //         };
    //         let result = drone.drone_movement(
    //             drone.location.clone(),
    //             0.005,
    //             drone.drone_config.get_movement_rate(),
    //             target_location,
    //         );

    //         assert!(result.is_ok());
    //         let new_location = drone.location;

    //         let distance_from_center =
    //             ((new_location.lat - 0.0).powi(2) + (new_location.long - 0.0).powi(2)).sqrt();
    //         assert!(distance_from_center <= 0.005);
    //     });
    //     handle.join().unwrap();
    // }

    /// Esta función de prueba simula el comportamiento de un dron cuando su batería se descarga.
    ///
    /// La prueba comienza creando un nuevo broker y ejecutándolo en un hilo separado.
    /// Luego, crea un dron en una ubicación específica e inicia un hilo para simular la descarga de la batería.
    ///
    /// En otro hilo, el dron se mueve hacia una ubicación objetivo hasta que su nivel de batería es bajo.
    /// Cuando el nivel de batería del dron es bajo, verifica que la ubicación del dron no sea la misma que la ubicación objetivo.
    /// Esto se debe a que el dron deberia estar en camino o yendo hacia el centro de carga debido a su nivel de bateria
    ///
    /// Después de que el nivel de batería del dron es bajo, deja de mover el dron y verifica que la ubicación del dron sea la misma que la ubicación objetivo.
    /// Esto se debe a que el dron debería haber regresado a la ubicación objetivo después de que su batería se descargó.
    ///
    #[test]
    fn test_volver_a_pos_original_despues_de_cargarse() {
        let args = vec!["127.0.0.1".to_string(), "5012".to_string()];
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
            let location = location::Location::new(latitude, longitude);
            let center_location = location::Location::new(0.0, 0.0);
            let drone = Drone::new(
                1,
                location,
                center_location,
                "./src/drones/drone_config.json",
                "127.0.0.1:5012".to_string(),
            )
            .unwrap();

            //thread de descarga de bateria
            let mut drone_clone = drone.clone();
            let _ = thread::spawn(move || {
                let _ = drone_clone.battery_discharge();
            });

            let mut drone_clone = drone.clone();
            let drone_clone2 = drone.clone();
            let _ = thread::spawn(move || {
                let mut terminado = false;
                let target_location = location::Location::new(0.001, 0.001);
                let _ = drone.location.clone();

                while !terminado {
                    let target_location = location::Location::new(0.001, 0.001);
                    let _ = drone_clone.location.clone();
                    let radius: f64 = 0.005;

                    let _ = drone_clone.drone_movement(
                        drone_clone.location.clone(),
                        radius,
                        100,
                        target_location.clone(),
                    );
                    if drone_clone2.clone().get_state() == DroneState::LowBatteryLevel {
                        assert_ne!(drone_clone.location, target_location.clone());
                        terminado = true;
                    }
                }
                assert_eq!(drone_clone.location, target_location.clone());
            });
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
        let config_file_path = "./src/drones/drone_config.json";
        let address = addres.to_string();

        Drone::new(1, location, center_location, config_file_path, address).unwrap()
    }
}
