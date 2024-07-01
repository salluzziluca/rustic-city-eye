use std::fmt;

/// Errores que se pueden lanzar desde el software de control de Drones.
#[derive(Debug, PartialEq)]
pub enum DroneError {
    ReadingConfigFileError,
    DroneCenterNotFound,
    ProtocolError(String),
    BatteryEmpty,
    SubscribeError(String),
    LockError(String),
    ChargingBatteryError(String),
    PatrollingError(String),
    MovingToIncidentError(String),
    ReceiveError(String),
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
            DroneError::BatteryEmpty => write!(
                f,
                "Error: la bateria del dron está completamente descargada."
            ),
            DroneError::SubscribeError(ref err) => write!(f, "Error de protocolo: {}", err),
            DroneError::LockError(ref err) => write!(f, "Error al adquirir un lock: {}", err),
            DroneError::ChargingBatteryError(ref err) => {
                write!(f, "Error al cargar la bateria del Drone: {}", err)
            }
            DroneError::PatrollingError(ref err) => write!(f, "Error al patrullar: {}", err),
            DroneError::MovingToIncidentError(ref err) => {
                write!(f, "Error al moverse al incidente: {}", err)
            }
            DroneError::ReceiveError(ref err) => write!(f, "Error al recibir mensaje: {}", err),
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
