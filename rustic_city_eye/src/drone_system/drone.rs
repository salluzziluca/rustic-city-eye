use std::sync::mpsc;

use super::{drone_config::DroneConfig, drone_error::DroneError, drone_state::DroneState};

#[derive(Debug)]
pub struct Drone {
    /// latitude y longitude nos indican la posicion actual del Drone.
    latitude: f64,
    longitude: f64,

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
}

impl Drone {
    pub fn new(latitude: f64, longitude: f64) -> Result<Drone, DroneError> {
        let drone_config =
            match DroneConfig::read_drone_config("./src/drone_system/drone_config.json") {
                Ok(c) => c,
                Err(err) => return Err(err),
            };

        Ok(Drone {
            longitude,
            latitude,
            drone_config,
            drone_state: DroneState::Waiting,
        })
    }

    /// Pongo a correr el Drone. Ira descargando su bateria dependiendo de
    /// su configuracion. Una vez que se descarga, su estado para a ser de Low Battery Level,
    /// y va a proceder a moverse hacia su central.
    pub fn run_drone(&mut self) {
        let (tx, rx) = mpsc::channel();
        let drone_state = self
            .drone_config
            .run_drone(self.latitude, self.longitude, tx);

        self.drone_state = drone_state;

        let (new_latitude, new_longitude) = rx.try_recv().unwrap();

        self.latitude = new_latitude;
        self.longitude = new_longitude;

        println!("lat: {} long: {}", self.latitude, self.longitude);
    }

    pub fn get_state(self) -> DroneState {
        self.drone_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_01_drone_creation_ok() {
    //     let latitude = 1.1;
    //     let longitude = 12.1;
    //     let drone_config =
    //         DroneConfig::read_drone_config("./src/drone_system/drone_config.json").unwrap();

    //     let drone = Drone::new(latitude, longitude).unwrap();

    //     assert_eq!(
    //         Drone {
    //             longitude: 12.1,
    //             latitude: 1.1,
    //             drone_config,
    //             drone_state: DroneState::Waiting
    //         },
    //         drone
    //     );
    // }

    #[test]
    fn test_02_drone_low_battery_level_state_ok() {
        let latitude = 0.0;
        let longitude = 0.0;
        let mut drone = Drone::new(latitude, longitude).unwrap();

        drone.run_drone();

        assert_eq!(drone.get_state(), DroneState::LowBatteryLevel);
    }
}
