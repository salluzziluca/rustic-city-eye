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
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DroneConfig {
    /// Indica el nivel de bateria del Drone. Se va descargando con el paso del tiempo.
    battery_level: Arc<RwLock<i64>>,

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
    pub fn run_drone(
        &mut self,
        location: Location,
        location_sender: Sender<Location>,
    ) -> DroneState {
        let battery_clone_one = self.battery_level.clone();
        let battery_clone_two = self.battery_level.clone();
        let battery_discharge_rate = self.battery_discharge_rate_milisecs;
        let radius = self.operation_radius;
        let movement_rate = self.movement_rate;

        let discharge_battery = std::thread::spawn(move || {
            DroneConfig::drone_battery_discharge(battery_clone_one, battery_discharge_rate);
        });

        let move_drone = std::thread::spawn(move || {
            DroneConfig::drone_movement(
                location,
                battery_clone_two,
                radius,
                location_sender,
                movement_rate,
            );
        });

        let _ = discharge_battery.join();
        let _ = move_drone.join();
        DroneState::LowBatteryLevel
    }

    /// closure del thread de descarga de bateria del Drone.
    ///
    /// Lo que se hace es ir descargando el nivel de bateria del
    /// Drone segun indique la tasa de descarga de bateria del mismo(definida
    /// en la config del Drone).
    ///
    /// Cada vez que se cumpla "un ciclo" de la tasa de descarga, se reduce la bateria del
    /// Drone en un 1%.
    fn drone_battery_discharge(battery_level: Arc<RwLock<i64>>, battery_discharge_rate: i64) {
        let mut last_discharge_time = Utc::now();

        loop {
            let mut lock = match battery_level.write() {
                Ok(lock) => lock,
                Err(_) => {
                    println!("Failed to acquire write lock. Exiting thread.");
                    break;
                }
            };
            let current_time = Utc::now();
            let elapsed_time = current_time
                .signed_duration_since(last_discharge_time)
                .num_milliseconds();

            if elapsed_time >= battery_discharge_rate {
                *lock -= 1;

                if *lock < 0 {
                    *lock = 0;
                    break;
                }
                last_discharge_time = current_time;
            }
        }
    }

    /// closure del thread de movimiento del Drone.
    ///
    /// Lo que se hace es generar una direccion de movimiento(utilizando rand),
    /// y se intenta generar la nueva posicion, siempre respetando el area de operacion
    /// del Drone.
    ///
    /// Si se generase una posicion que esta por fuera del area, se van a
    /// generar las direcciones aleatorias que sean necesarias hasta obtener una que nos
    /// lleve de nuevo adentro del area.
    ///
    /// Se utiliza la tasa de movimiento del Drone, que viene definida en la configuracion:
    /// la idea es que el Drone se mueva cada cierto intervalo de tiempo definido por esta tasa de movimiento.
    fn drone_movement(
        location: Location,
        battery_level: Arc<RwLock<i64>>,
        radius: f64,
        location_sender: Sender<Location>,
        movement_rate: i64,
    ) {
        // cuando se pone a correr al drone, se toma a su posicion inicial
        // como el centro de su area de operacion(es un circulo!)
        let center_lat = location.lat;
        let center_long = location.long;

        let mut current_lat = location.lat;
        let mut current_long = location.long;
        let mut last_move_time = Utc::now();
        let mut rng = rand::thread_rng();

        loop {
            let lock = match battery_level.read() {
                Ok(lock) => lock,
                Err(_) => {
                    println!("Failed to acquire read lock. Exiting thread.");
                    break;
                }
            };
            if *lock > 0 {
                let current_time = Utc::now();
                let elapsed_time = current_time
                    .signed_duration_since(last_move_time)
                    .num_milliseconds();

                if elapsed_time >= movement_rate {
                    let mut new_lat;
                    let mut new_long;

                    loop {
                        let delta_lat: f64 = rng.gen_range(-radius..radius);
                        let delta_long: f64 = rng.gen_range(-radius..radius);

                        new_lat = current_lat + delta_lat;
                        new_long = current_long + delta_long;

                        let distance_from_center = ((new_lat - center_lat).powi(2)
                            + (new_long - center_long).powi(2))
                        .sqrt();
                        if distance_from_center <= radius {
                            break;
                        }
                    }

                    current_lat = new_lat;
                    current_long = new_long;

                    let location = location::Location::new(new_lat, new_long);

                    let _ = location_sender.send(location);

                    last_move_time = current_time;
                }
            } else {
                break;
            }
        }
    }

    /// Carga al Drone de acuerdo a la tasa de carga que venga definida en la configuracion.
    ///
    /// Al llegar al 100%, devuelve un DroneState del tipo Waiting.
    pub fn charge_battery(&mut self) -> Result<DroneState, DroneError> {
        let mut start_time = Utc::now();

        loop {
            let mut lock = match self.battery_level.write() {
                Ok(lock) => lock,
                Err(_) => {
                    return Err(DroneError::ReadingConfigFileError);
                }
            };
            let current_time = Utc::now();
            let elapsed_time = current_time
                .signed_duration_since(start_time)
                .num_milliseconds();

            if elapsed_time >= self.battery_charge_rate_milisecs {
                *lock += 1;

                if *lock > 100 {
                    *lock = 100;
                    return Ok(DroneState::Waiting);
                }
                start_time = current_time;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn test_01_config_creation_cases() {
        let config_ok = DroneConfig::new("./src/drone_system/drone_config.json");

        let config_err = DroneConfig::new("este/es/un/path/feo");

        assert!(config_ok.is_ok());
        assert!(config_err.is_err());
    }

    #[test]
    fn test_02_running_drone_and_discharges_battery_ok() -> std::io::Result<()> {
        let (tx, _rx) = mpsc::channel();
        let mut config =
            DroneConfig::read_drone_config("./src/drone_system/drone_config.json").unwrap();
        let location = location::Location::new(1.1, 12.1);
        let final_drone_state = config.run_drone(location, tx);

        assert_eq!(final_drone_state, DroneState::LowBatteryLevel);

        Ok(())
    }

    #[test]
    fn test_03_drone_charge_ok() {
        let mut config = DroneConfig::read_drone_config("./tests/drone_config_test.json").unwrap();

        let final_state = config.charge_battery().unwrap();

        assert_eq!(final_state, DroneState::Waiting);
    }

    #[test]
    fn test_04_bad_config_file() {
        let config = DroneConfig::read_drone_config("este/es/un/path/feo");

        assert!(config.is_err());
    }
}
