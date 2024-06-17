use chrono::Utc;
use rand::Rng;
use serde::Deserialize;

use std::{
    fs::File,
    io::BufReader,
    sync::{mpsc::Sender, Arc, RwLock},
};

use crate::utils::location::{self, Location};

use super::{drone_error::DroneError, drone_state::DroneState};

/// Sirve para levantar la configuracion del Drone a partir del JSON.
/// Pone a correr al Drone:
///     - Simula su descarga de bateria.
///     - Hace que se mueva dentro de su area de operacion.
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct DroneConfig {
    /// Indica el nivel de bateria del Drone. Se va descargando con el paso del tiempo.

    /// Indice la tasa de carga de la bateria en milisegundos.
    /// Por ej: si vale 10, por cada segundo que pase, la
    /// bateria del Drone aumentara en un 100 por ciento.
    battery_charge_rate_milisecs: i64,

    /// Indice la tasa de desgaste de la bateria en milisegundos.
    /// Por ej: si vale 10, por cada segundo que pase, la
    /// bateria del Drone se reducira en un 100 por ciento.
    battery_discharge_rate_milisecs: i64,

    /// El Drone circulara en un area de operacion determinado por el archivo de configuracion.
    /// A medida que pasa el tiempo, el Drone va moviendose dentro de ese area.
    operation_radius: f64,

    /// Es la velocidad con la que el Drone va a circular.
    /// Para simplificarle la vida al usuario, el valor que se
    /// lee desde el archivo de configuracion esta en km/h.
    movement_rate: i64,
}

impl DroneConfig {
    /// Leo la configuracion a partir de un archivo json.
    pub fn new(config_file_path: &str) -> Result<DroneConfig, DroneError> {
        match DroneConfig::read_drone_config(config_file_path) {
            Ok(config) => Ok(config),
            Err(err) => Err(err),
        }
    }

    /// Toma un path a un archivo de configuracion y levanta el DroneConfig.
    fn read_drone_config(file_path: &str) -> Result<DroneConfig, DroneError> {
        let config_file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => return Err(DroneError::ReadingConfigFileError),
        };

        let reader: BufReader<File> = BufReader::new(config_file);
        let config = match serde_json::from_reader(reader) {
            Ok(c) => c,
            Err(_) => return Err(DroneError::ReadingConfigFileError),
        };

        Ok(config)
    }

    /// Simula la descarga de bateria del Drone, dependiendo de su
    /// tasa de descarga en milisegundos.
    ///
    /// Tambien, el Drone se movera dependiendo del tiempo
    /// transcurrido, su velocidad asignada y su radio de operacion.
    ///
    /// Para poder hacer ambas cosas a la vez, trabajo con dos threads: uno encargado de descargar la bateria,
    /// y otro que se encarga de mover al Drone(siempre y cuando tenga bateria).

    pub fn get_battery_discharge_rate(&self) -> i64 {
        self.battery_discharge_rate_milisecs
    }

    pub fn get_battery_charge_rate_milisecs(&mut self) -> i64 {
        self.battery_charge_rate_milisecs
    }
    pub fn get_operation_radius(&self) -> f64 {
        self.operation_radius
    }

    pub fn get_movement_rate(&self) -> i64 {
        self.movement_rate
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn test_01_config_creation_cases() {
        let config_ok = DroneConfig::new("./src/drones/drone_config.json");

        let config_err = DroneConfig::new("este/es/un/path/feo");

        assert!(config_ok.is_ok());
        assert!(config_err.is_err());
    }

    // #[test]
    // fn test_02_running_drone_and_discharges_battery_ok() -> std::io::Result<()> {
    //     let (tx, _rx) = mpsc::channel();
    //     let mut config = DroneConfig::read_drone_config("./src/drones/drone_config.json").unwrap();
    //     let location = location::Location::new(1.1, 12.1);
    //     let final_drone_state = config.run_drone(location, tx);

    //     assert_eq!(final_drone_state, DroneState::LowBatteryLevel);

    //     Ok(())
    // }

    // #[test]
    // fn test_03_drone_charge_ok() {
    //     let mut config = DroneConfig::read_drone_config("./tests/drone_config_test.json").unwrap();

    //     let final_state = config.charge_battery().unwrap();

    //     assert_eq!(final_state, DroneState::Waiting);
    // }

    #[test]
    fn test_04_bad_config_file() {
        let config = DroneConfig::read_drone_config("este/es/un/path/feo");

        assert!(config.is_err());
    }
}
