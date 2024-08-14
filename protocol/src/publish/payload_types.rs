use std::{
    any::Any,
    io::{Error, ErrorKind, Read},
};

use serde::{Deserialize, Serialize};
use utils::{
    camera::Camera, incident::Incident, incident_payload::IncidentPayload, location::Location,
    single_disconnect_payload::SingleDisconnectPayload,
};

use utils::{protocol_error::ProtocolError, reader::*, writer::*};

use super::payload::Payload;

/// Aqui se definen los distintos tipos de payload que va a soportar nuestra aplicacion.
/// La idea es que implemente el trait de Payload, de forma tal que sepa escribirse sobre un stream dado.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PayloadTypes {
    IncidentLocation(IncidentPayload),
    AttendingIncident(IncidentPayload),
    WillPayload(String),
    LocationPayload(Location),
    CamerasUpdatePayload(Vec<Camera>),
    DroneLocation(u32, Location, Location),
    SingleDroneDisconnect(SingleDisconnectPayload),
}

impl Payload for PayloadTypes {
    fn write_to(&self, stream: &mut dyn std::io::prelude::Write) -> Result<(), ProtocolError> {
        match self {
            PayloadTypes::IncidentLocation(payload) => {
                write_u8(stream, &1)?;
                payload.write_to(stream)?;

                Ok(())
            }
            PayloadTypes::WillPayload(payload) => {
                write_u8(stream, &2)?;
                write_string(stream, payload)?;

                Ok(())
            }
            PayloadTypes::LocationPayload(payload) => {
                write_u8(stream, &3)?;
                write_string(stream, &payload.get_latitude().to_string())?;
                write_string(stream, &payload.get_longitude().to_string())?;

                Ok(())
            }
            PayloadTypes::CamerasUpdatePayload(payload) => {
                write_u8(stream, &4)?;
                write_u8(stream, &(payload.len() as u8))?;

                for camera in payload {
                    let mut camera_clone = camera.clone();
                    match camera_clone.write_to(stream) {
                        Ok(_) => {}
                        Err(_) => return Err(ProtocolError::WriteError),
                    }
                }

                Ok(())
            }
            PayloadTypes::DroneLocation(drone_id, location, target_location) => {
                write_u8(stream, &5)?;
                write_string(stream, &drone_id.to_string())?;
                write_string(stream, &location.get_latitude().to_string())?;
                write_string(stream, &location.get_longitude().to_string())?;
                write_string(stream, &target_location.get_latitude().to_string())?;
                write_string(stream, &target_location.get_longitude().to_string())?;

                Ok(())
            }
            PayloadTypes::AttendingIncident(payload) => {
                write_u8(stream, &6)?;
                payload.write_to(stream)?;

                Ok(())
            }
            PayloadTypes::SingleDroneDisconnect(payload) => {
                write_u8(stream, &7)?;
                payload.write_to(stream)?;

                Ok(())
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PayloadTypes {
    /// Dependiendo del id del payload que se lea, se va a reconstruir el payload a partir de lo
    /// leido efectivamente del stream.
    pub fn read_from(stream: &mut dyn Read) -> Result<PayloadTypes, std::io::Error> {
        let payload_type_id = read_u8(stream)?;

        let payload_type = match payload_type_id {
            1 => {
                let longitude_string = read_string(stream)?;
                let long = match longitude_string.parse::<f64>() {
                    Ok(long) => long,
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Error while reading payload".to_string(),
                        ))
                    }
                };

                let latitude_string = read_string(stream)?;

                let lat = match latitude_string.parse::<f64>() {
                    Ok(lat) => lat,
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Error while reading payload".to_string(),
                        ))
                    }
                };

                let location = Location::new(lat, long);
                let incident = Incident::new(location);

                PayloadTypes::IncidentLocation(IncidentPayload::new(incident))
            }
            4 => {
                let lenght = read_u8(stream)?;
                let mut cameras = Vec::new();
                for _ in 0..lenght {
                    let camera = Camera::read_from(stream)?;
                    cameras.push(camera);
                }

                PayloadTypes::CamerasUpdatePayload(cameras)
            }
            5 => {
                let drone_id_string = read_string(stream)?;
                let drone_id = match drone_id_string.parse::<u32>() {
                    Ok(drone_id) => drone_id,
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Error while reading payload".to_string(),
                        ))
                    }
                };

                let longitude_string = read_string(stream)?;
                let long = match longitude_string.parse::<f64>() {
                    Ok(long) => long,
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Error while reading payload".to_string(),
                        ))
                    }
                };

                let latitude_string = read_string(stream)?;
                let lat = match latitude_string.parse::<f64>() {
                    Ok(lat) => lat,
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Error while reading payload".to_string(),
                        ))
                    }
                };

                let target_longitude_string = read_string(stream)?;
                let target_long = match target_longitude_string.parse::<f64>() {
                    Ok(long) => long,
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Error while reading payload".to_string(),
                        ))
                    }
                };

                let target_latitude_string = read_string(stream)?;
                let target_lat = match target_latitude_string.parse::<f64>() {
                    Ok(lat) => lat,
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Error while reading payload".to_string(),
                        ))
                    }
                };

                let target_location = Location::new(target_lat, target_long);
                let location = Location::new(lat, long);

                PayloadTypes::DroneLocation(drone_id, location, target_location)
            }
            6 => {
                let longitude_string = read_string(stream)?;
                let long = match longitude_string.parse::<f64>() {
                    Ok(long) => long,
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Error while reading payload".to_string(),
                        ))
                    }
                };

                let latitude_string = read_string(stream)?;

                let lat = match latitude_string.parse::<f64>() {
                    Ok(lat) => lat,
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Error while reading payload".to_string(),
                        ))
                    }
                };

                let location = Location::new(lat, long);
                let incident = Incident::new(location);
                PayloadTypes::AttendingIncident(IncidentPayload::new(incident))
            }
            7 => {
                let id = read_u32(stream)?;

                PayloadTypes::SingleDroneDisconnect(SingleDisconnectPayload::new(id))
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Error while reading payload".to_string(),
                ))
            }
        };
        Ok(payload_type)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::{Cursor, Read};


    #[test]
    fn test_write_to() {
        let location = Location::new(1.0, 2.0);
        let incident = Incident::new(location);
        let incident_payload = IncidentPayload::new(incident.clone());
        let payload = PayloadTypes::IncidentLocation(incident_payload.clone());

        let mut cursor = Cursor::new(Vec::new());
        payload.write_to(&mut cursor).unwrap();
        cursor.set_position(0);

        let mut buf = [0u8; 1];
        cursor.read_exact(&mut buf).unwrap();
        assert_eq!(buf[0], 1);
    }

    #[test]
    fn test_as_any() {
        let location = Location::new(1.0, 2.0);
        let incident = Incident::new(location);
        let incident_payload = IncidentPayload::new(incident.clone());
        let payload = PayloadTypes::IncidentLocation(incident_payload.clone());

        assert!(payload.as_any().is::<PayloadTypes>());
    }

    #[test]
    fn test_clone() {
        let location = Location::new(1.0, 2.0);
        let incident = Incident::new(location);
        let incident_payload = IncidentPayload::new(incident.clone());
        let payload = PayloadTypes::IncidentLocation(incident_payload.clone());

        assert_eq!(payload.clone(), payload);
    }

    #[test]
    fn test_eq() {
        let location = Location::new(1.0, 2.0);
        let incident = Incident::new(location);
        let incident_payload = IncidentPayload::new(incident.clone());
        let payload = PayloadTypes::IncidentLocation(incident_payload.clone());

        assert_eq!(payload, payload);
    }

    #[test]
    fn test_single_drone_disconnect() {
        let disc_payload = SingleDisconnectPayload::new(1);
        let payload = PayloadTypes::SingleDroneDisconnect(disc_payload.clone());

        let mut cursor = Cursor::new(Vec::new());
        payload.write_to(&mut cursor).unwrap();
        cursor.set_position(0);

        let read_payload = PayloadTypes::read_from(&mut cursor).unwrap();
        assert_eq!(read_payload, payload);
    }
}
