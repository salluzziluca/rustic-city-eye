use chrono::Utc;
use serde::Deserialize;

use std::{fs::File, io::BufReader};

use super::{drone_error::DroneError, drone_state::DroneState};

/// Sirve para levantar la configuracion del Drone a partir del JSON.
/// Pone a correr al Drone:
///     - Simula su descarga de bateria.
#[derive(Debug, Deserialize, PartialEq)]
pub struct DroneConfig {
    /// Indica el nivel de bateria del Drone. Se va descargando con el paso del tiempo.
    battery_level: i64,

    /// Indice la tasa de desgaste de la bateria en milisegundos.
    /// Por ej: si vale 10, por cada 10 segundos que pasen, la
    /// bateria del Drone se reducira en un 1 por ciento.
    battery_discharge_rate_milisecs: i64,

    /// El Drone circulara en un area de operacion determinado por el archivo de configuracion.
    /// A medida que pasa el tiempo, el Drone va moviendose dentro de ese area.
    operation_radius: usize,
}

impl DroneConfig {
    /// Toma un path a un archivo de configuracion y levanta el DroneConfig.
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

        Ok(config)
    }

    /// Simula la descarga de bateria del Drone, dependiendo de su
    /// tasa de descarga en milisegundos.
    pub fn run_drone(&mut self) -> DroneState {
        let mut start_time = Utc::now();

        while self.battery_level > 0 {
            let current_time = Utc::now();
            let elapsed = current_time
                .signed_duration_since(start_time)
                .num_milliseconds();

            if elapsed >= self.battery_discharge_rate_milisecs {
                self.battery_level -= 1;

                if self.battery_level < 0 {
                    self.battery_level = 0;
                    break;
                }

                start_time = current_time;
            }
        }

        DroneState::LowBatteryLevel
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
                battery_discharge_rate_milisecs: 50,
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

    #[test]
    fn test_03_running_drone_ok() {
        let mut config =
            DroneConfig::read_drone_config("./src/drone_system/drone_config.json").unwrap();

        let final_drone_state = config.run_drone();

        assert_eq!(final_drone_state, DroneState::LowBatteryLevel);
    }
}
