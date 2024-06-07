use chrono::Utc;
use rand::Rng;
use serde::Deserialize;

use std::{
    f64::consts::PI,
    fs::File,
    io::BufReader,
    sync::{mpsc::Sender, Arc, RwLock},
};

use super::{drone_error::DroneError, drone_state::DroneState};

/// Sirve para levantar la configuracion del Drone a partir del JSON.
/// Pone a correr al Drone:
///     - Simula su descarga de bateria.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DroneConfig {
    /// Indica el nivel de bateria del Drone. Se va descargando con el paso del tiempo.
    battery_level: Arc<RwLock<i64>>,

    /// Indice la tasa de desgaste de la bateria en milisegundos.
    /// Por ej: si vale 10, por cada 10 segundos que pasen, la
    /// bateria del Drone se reducira en un 1 por ciento.
    battery_discharge_rate_milisecs: i64,

    /// El Drone circulara en un area de operacion determinado por el archivo de configuracion.
    /// A medida que pasa el tiempo, el Drone va moviendose dentro de ese area.
    operation_radius: f64,

    /// Es la velocidad con la que el Drone va a circular.
    /// Para simplificarle la vida al usuario, el valor que se
    /// lee desde el archivo de configuracion esta en km/h.
    speed: f64,
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
    /// tasa de descarga en milisegundos. Tambien, el Drone se movera dependiendo del tiempo
    /// transcurrido, su velocidad asignada y su radio de operacion. Para poder hacer ambas cosas a la
    /// vez, trabajo con dos threads: uno encargado de descargar la bateria, y otro que se encarga de mover
    /// al Drone(siempre y cuando tenga bateria).
    pub fn run_drone(
        &mut self,
        latitude: f64,
        longitude: f64,
        location_sender: Sender<(f64, f64)>,
    ) -> DroneState {
        let battery_clone = self.battery_level.clone();
        let battery_clone_two = battery_clone.clone();
        let battery_discharge_rate = self.battery_discharge_rate_milisecs;
        let radius = self.operation_radius;

        // cuando se pone a correr al drone, se toma a su posicion inicial
        // como el centro de su area de operacion(es un circulo!)
        let center_lat = latitude;
        let center_long = longitude;


        let discharge_battery = std::thread::spawn(move || {
            let mut start_time = Utc::now();

            loop {
                let mut lock = match battery_clone.write() {
                    Ok(lock) => lock,
                    Err(_) => {
                        println!("Failed to acquire write lock. Exiting thread.");
                        break;
                    }
                };
                let current_time = Utc::now();
                let elapsed_time = current_time
                    .signed_duration_since(start_time)
                    .num_milliseconds();

                if elapsed_time >= battery_discharge_rate {
                    println!("discharging");
                    *lock -= 1;

                    if *lock < 0 {
                        *lock = 0;
                        break;
                    }
                    start_time = current_time;
                }
            }
        });

        let move_drone = std::thread::spawn(move || {
            let mut current_lat = latitude;
            let mut current_long = longitude;
            let mut last_move_time = Utc::now();
            let mut rng = rand::thread_rng();
    
            loop {
                let lock = battery_clone_two.read().unwrap();
                if *lock > 0 {
                    let current_time = Utc::now();
                    let elapsed_time = current_time
                        .signed_duration_since(last_move_time)
                        .num_milliseconds();
    
                    if elapsed_time >= 50 {
                        let mut new_lat;
                        let mut new_long;

                        loop {
                            let delta_lat: f64 = rng.gen_range(-radius..radius);
                            let delta_long: f64 = rng.gen_range(-radius..radius);

                            new_lat = current_lat + delta_lat;
                            new_long = current_long + delta_long;

                            let distance_from_center = ((new_lat - center_lat).powi(2) + (new_long - center_long).powi(2)).sqrt();
                            if distance_from_center <= radius {
                                break;
                            } else {
                                println!("Generated move out of bounds: ({}, {}). Regenerating...", new_lat, new_long);
                            }
                        }

                        current_lat = new_lat;
                        current_long = new_long;

                        println!("Moving to ({}, {})", new_lat, new_long);
                        let _ = location_sender.send((new_lat, new_long));
    
                        last_move_time = current_time;
                    }
                } else {
                    break;
                }
            }
        });

        let _ = discharge_battery.join();
        let _ = move_drone.join();
        DroneState::LowBatteryLevel
    }

    pub fn move_drone(&mut self, mut latitude: f64, mut longitude: f64, elapsed_time: i64) {
        let mut rng = rand::thread_rng();

        let elapsed_time_in_hours = elapsed_time as f64 / 3600000.0;
        let distance = self.speed * elapsed_time_in_hours;

        let angle = rng.gen_range(0.0..2.0 * PI);

        let distance_in_degrees = distance / 111.0; //representa a un grado de latitud/longitud.

        let delta_latitude = distance_in_degrees * angle.cos();
        let delta_longitude = distance_in_degrees * angle.sin();

        latitude += delta_latitude;
        longitude += delta_longitude;

        println!("new lat: {} new long: {}", latitude, longitude);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn test_01_config_creation_cases() {
        let config_ok = DroneConfig::read_drone_config("./src/drone_system/drone_config.json");

        let config_err = DroneConfig::read_drone_config("este/es/un/path/feo");

        assert!(config_ok.is_ok());
        assert!(config_err.is_err());
    }

    #[test]
    fn test_02_running_drone_ok() -> std::io::Result<()> {
        let (tx, rx) = mpsc::channel();
        let mut config =
            DroneConfig::read_drone_config("./src/drone_system/drone_config.json").unwrap();

        let final_drone_state = config.run_drone(1.1, 12.1, tx);

        let new_location = rx.try_recv().unwrap();

        assert_eq!(new_location, (10.0, 10.0));
        assert_eq!(final_drone_state, DroneState::LowBatteryLevel);

        Ok(())
    }
}
