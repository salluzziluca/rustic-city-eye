use std::fmt;

/// Errores que se pueden lanzar desde el software de control de Drones.
#[derive(Debug)]
pub enum DroneError {
    ReadingConfigFileError,
}

impl fmt::Display for DroneError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DroneError::ReadingConfigFileError => {
                write!(
                    f,
                    "Error: no se ha podido encontrar el archivo de configuracion de drones."
                )
            }
        }
    }
}
