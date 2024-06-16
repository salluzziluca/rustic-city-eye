use std::sync::mpsc;

use super::{drone_config::DroneConfig, drone_error::DroneError, drone_state::DroneState};
use crate::{
    mqtt::{
        client::Client,
        client_message, messages_config,
        publish::{
            publish_config::PublishConfig,
            publish_properties::{PublishProperties, TopicProperties},
        },
    },
    utils::{location::Location, payload_types::PayloadTypes},
};
#[derive(Debug)]
pub struct Drone {
    // ID unico para cada Drone.
    id: u32,
    ///posicion actual del Drone.
    location: Location,

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

    send_to_client_channel: mpsc::Sender<Box<dyn messages_config::MessagesConfig + Send>>,
}

impl Drone {
    /// levanto su configuracion, y me guardo su posicion inicial.
    pub fn new(
        id: u32,
        location: Location,
        config_file_path: &str,
        address: String,
    ) -> Result<Drone, DroneError> {
        let drone_config = DroneConfig::new(config_file_path)?;
        let connect_config = match client_message::Connect::read_connect_config(
            "./src/drones/connect_config.json",
        ) {
            Ok(config) => config,
            Err(e) => return Err(DroneError::ProtocolError(e.to_string())),
        };
        let (tx, rx) = mpsc::channel();

        let drone_client = match Client::new(rx, address, connect_config) {
            Ok(client) => client,
            Err(e) => return Err(DroneError::ProtocolError(e.to_string())),
        };
        Ok(Drone {
            id,
            location,
            drone_config,
            drone_state: DroneState::Waiting,
            drone_client,
            send_to_client_channel: tx,
        })
    }

    /// Pongo a correr el Drone. Ira descargando su bateria dependiendo de
    /// su configuracion. Una vez que se descarga, su estado para a ser de Low Battery Level,
    /// y va a proceder a moverse hacia su central.
    pub fn run_drone(&mut self) -> Result<(), DroneError> {
        let (tx, rx) = mpsc::channel();
        let drone_state = self.drone_config.run_drone(self.location.clone(), tx);
        match self.drone_client.client_run() {
            Ok(client) => client,
            Err(e) => return Err(DroneError::ProtocolError(e.to_string())),
        };

        while let Ok(location) = rx.recv() {
            self.location = location;
            let payload = PayloadTypes::LocationPayload(self.location.clone());
            let topic_properties = TopicProperties {
                topic_alias: 10,
                response_topic: "String".to_string(),
            };

            let properties = PublishProperties::new(
                1,
                10,
                topic_properties,
                [1, 2, 3].to_vec(),
                "a".to_string(),
                1,
                "a".to_string(),
            );

            let publish_config =
                PublishConfig::new(1, 1, 0, "incidente".to_string(), payload, properties);

            let _ = self.send_to_client_channel.send(Box::new(publish_config));

            self.drone_state = drone_state.clone();

            if self.drone_state == DroneState::LowBatteryLevel {
                self.charge_drone()?;
            }
        }
        Ok(())
    }

    pub fn charge_drone(&mut self) -> Result<(), DroneError> {
        let new_state = match self.drone_config.charge_battery() {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        self.drone_state = new_state;
        Ok(())
    }

    pub fn get_state(self) -> DroneState {
        self.drone_state
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use crate::{mqtt::broker::Broker, utils::location};

    use super::*;

    #[test]
    fn test_01_drone_low_battery_level_state_ok() {
        let args = vec!["127.0.0.1".to_string(), "5000".to_string()];
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
            let latitude = 0.0;
            let longitude = 0.0;
            let location = location::Location::new(latitude, longitude);
            let mut drone = Drone::new(
                1,
                location,
                "./src/drones/drone_config.json",
                "127.0.0.1:5000".to_string(),
            )
            .unwrap();

            let _ = drone.run_drone();

            assert_eq!(drone.get_state(), DroneState::Waiting);
        });
    }

    #[test]
    fn test_02_drone_going_to_charge_battery_ok() {
        let args = vec!["127.0.0.1".to_string(), "5000".to_string()];
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
            let latitude = 0.0;
            let longitude = 0.0;
            let location = location::Location::new(latitude, longitude);
            let mut drone = Drone::new(
                1,
                location,
                "./tests/drone_config_test.json",
                "127.0.0.1:5000".to_string(),
            )
            .unwrap();

            let _ = drone.run_drone();

            assert_eq!(drone.get_state(), DroneState::Waiting);
        });
    }

    #[test]
    fn test_03_get_id_ok() {
        let args = vec!["127.0.0.1".to_string(), "5000".to_string()];
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
            let latitude = 0.0;
            let longitude = 0.0;
            let location = location::Location::new(latitude, longitude);
            let drone = Drone::new(
                1,
                location,
                "./tests/drone_config_test.json",
                "127.0.0.1:5000".to_string(),
            )
            .unwrap();

            assert_eq!(drone.get_id(), 1);
        });
    }

    #[test]
    fn test_04_drone_bad_config_file() {
        let latitude = 0.0;
        let longitude = 0.0;
        let location = location::Location::new(latitude, longitude);
        let drone = Drone::new(
            1,
            location,
            "./tests/bad_config_file.json",
            "127.0.0.1:5000".to_string(),
        );
        assert!(drone.is_err());
    }
}
