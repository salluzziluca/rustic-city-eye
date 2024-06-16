use std::fmt;

/// Errores que se pueden lanzar desde el software de control de Drones.
#[derive(Debug, PartialEq)]
pub enum DroneError {
    ReadingConfigFileError,
    DroneCenterNotFound,
    ProtocolError(String),
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
            DroneError::DroneCenterNotFound => {
                write!(f, "Error: no se ha podido encontrar el centro de drones.")
            }
            DroneError::ProtocolError(ref err) => write!(f, "Error de protocolo: {}", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drone_error_display() {
        let error = DroneError::ReadingConfigFileError;
        assert_eq!(
            format!("{}", error),
            "Error: no se ha podido encontrar el archivo de configuracion de drones."
        );

        let error = DroneError::DroneCenterNotFound;
        assert_eq!(
            format!("{}", error),
            "Error: no se ha podido encontrar el centro de drones."
        );

        let error = DroneError::ProtocolError("Error de protocolo".to_string());
        assert_eq!(
            format!("{}", error),
            "Error de protocolo: Error de protocolo"
        );
    }
}
