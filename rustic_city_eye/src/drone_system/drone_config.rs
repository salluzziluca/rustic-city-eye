use serde::Deserialize;

use std::{fs::File, io::BufReader};

use super::drone_error::DroneError;

#[derive(Debug, Deserialize, PartialEq)]
pub struct DroneConfig {
    /// Indica el nivel de bateria del Drone. Se va descargando con el paso del tiempo.
    battery_level: usize,

    /// El Drone circulara en un area de operacion determinado por el archivo de configuracion.
    /// A medida que pasa el tiempo, el Drone va moviendose dentro de ese area.
    operation_radius: usize,
}

impl DroneConfig {
    pub fn read_drone_config(file_path: &str) -> Result<DroneConfig, DroneError> {
        let config_file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => return Err(DroneError::ReadingConfigFileError),
        };

        let reader: BufReader<File> = BufReader::new(config_file);
        let config = match serde_json::from_reader(reader) {
            Ok(c) => c,
            Err(_) => return Err(DroneError::ReadingConfigFileError),
        };

        println!("config {:?}", config);

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_config_reading_ok() {
        let config =
            DroneConfig::read_drone_config("./src/drone_system/drone_config.json").unwrap();

        assert_eq!(
            DroneConfig {
                battery_level: 100,
                operation_radius: 3
            },
            config
        );
    }

    #[test]
    fn test_02_config_reading_json_err() {
        let config = DroneConfig::read_drone_config("este/es/un/path/feo");

        assert!(config.is_err());
    }
}
