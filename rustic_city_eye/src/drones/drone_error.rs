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
    SendError(String),
    CentralError(String),
}

impl fmt::Display for DroneError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DroneError::ReadingConfigFileError => {
                write!(
                    f,
                    "Error: The drone configuration file could not be found."
                )
            }
            DroneError::DroneCenterNotFound => {
                write!(f, "Error: The drone center could not be found.")
            }
            DroneError::ProtocolError(ref err) => write!(f, "Protocol error: {}", err),
            DroneError::BatteryEmpty => write!(
                f,
                "Error: the drone battery is completely discharged."
            ),
            DroneError::SubscribeError(ref err) => write!(f, "Protocol error: {}", err),
            DroneError::LockError(ref err) => write!(f, "Error acquiring a lock: {}", err),
            DroneError::ChargingBatteryError(ref err) => {
                write!(f, "Error when charging the Drone battery: {}", err)
            }
            DroneError::PatrollingError(ref err) => write!(f, "Error while patrolling: {}", err),
            DroneError::MovingToIncidentError(ref err) => {
                write!(f, "Error moving to incident: {}", err)
            }
            DroneError::ReceiveError(ref err) => write!(f, "Error receiving message: {}", err),
            DroneError::SendError(ref err) => write!(f, "Error sending message: {}", err),
            DroneError::CentralError(ref err) => write!(f, "Error in the drone center {}", err),
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
            "Error: The drone configuration file could not be found."
        );

        let error = DroneError::DroneCenterNotFound;
        assert_eq!(format!("{}", error), "Error: The drone center could not be found.");

        let error = DroneError::ProtocolError("Error".to_string());
        assert_eq!(format!("{}", error), "Protocol error: Error");
        
    }
}
