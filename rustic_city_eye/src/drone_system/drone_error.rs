use std::fmt;

///Here are detailed all the errors that the protocol is capable of throwing.
/// Unspecified se usa de placeholder para los results de los tests
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
