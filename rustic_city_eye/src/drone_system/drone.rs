use std::sync::mpsc;

use super::{drone_config::DroneConfig, drone_error::DroneError, drone_state::DroneState};

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

        Ok(Drone {
            id,
            location,
            drone_config,
            drone_state: DroneState::Waiting,
        })
    }

    /// Pongo a correr el Drone. Ira descargando su bateria dependiendo de
    /// su configuracion. Una vez que se descarga, su estado para a ser de Low Battery Level,
    /// y va a proceder a moverse hacia su central.
    pub fn run_drone(&mut self) -> Result<(), DroneError> {
        let (tx, rx) = mpsc::channel();
        let drone_state = self
            .drone_config
            .run_drone(self.latitude, self.longitude, tx);

        while let Ok((new_latitude, new_longitude)) = rx.recv() {
            self.latitude = new_latitude;
            self.longitude = new_longitude;
        }
        self.drone_state = drone_state;

        if self.drone_state == DroneState::LowBatteryLevel {
            self.charge_drone()?;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_drone_low_battery_level_state_ok() {
        let latitude = 0.0;
        let longitude = 0.0;
        let mut drone =
            Drone::new(latitude, longitude, "./src/drone_system/drone_config.json").unwrap();

        let _ = drone.run_drone();

        assert_eq!(drone.get_state(), DroneState::Waiting);
    }

    #[test]
    fn test_02_drone_going_to_charge_battery_ok() {
        let latitude = 0.0;
        let longitude = 0.0;
        let mut drone = Drone::new(latitude, longitude, "./tests/drone_config_test.json").unwrap();

        let _ = drone.run_drone();

        assert_eq!(drone.get_state(), DroneState::Waiting);
    }
}
