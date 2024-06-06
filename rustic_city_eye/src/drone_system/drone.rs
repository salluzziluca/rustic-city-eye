use super::{drone_config::DroneConfig, drone_error::DroneError};

#[derive(Debug, PartialEq)]
pub struct Drone {
    /// latitude y longitude nos indican la posicion actual del Drone.
    latitude: f64,
    longitude: f64,

    /// La configuracion del Drone contiene el nivel de bateria del mismo y
    /// el radio de operacion.
    drone_config: DroneConfig,
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_drone_creation_ok() {
        let latitude = 1.1;
        let longitude = 12.1;
        let drone_config =
            DroneConfig::read_drone_config("./src/drone_system/drone_config.json").unwrap();

        let drone = Drone::new(latitude, longitude).unwrap();

        assert_eq!(
            Drone {
                longitude: 12.1,
                latitude: 1.1,
                drone_config
            },
            drone
        );
    }
}
