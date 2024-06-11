use std::{
    any::Any,
    io::{Error, ErrorKind},
};

use crate::{
    monitoring::incident::Incident,
    mqtt::{payload::Payload, protocol_error::ProtocolError},
    utils::{
        incident_payload::IncidentPayload,
        location::Location,
        reader::{read_string, read_u8},
    },
};

/// Aqui se definen los distintos tipos de payload que va a soportar nuestra aplicacion.
/// La idea es que implemente el trait de Payload, de forma tal que sepa escribirse sobre un stream dado.
#[derive(Clone, Debug, PartialEq)]
pub enum PayloadTypes {
    IncidentLocation(IncidentPayload),
}

impl Payload for PayloadTypes {
    fn write_to(&self, stream: &mut dyn std::io::prelude::Write) -> Result<(), ProtocolError> {
        match self {
            PayloadTypes::IncidentLocation(payload) => {
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
    pub fn read_from(
        stream: &mut dyn std::io::prelude::Read,
    ) -> Result<PayloadTypes, std::io::Error> {
        let payload_type_id = read_u8(stream)?;

        let payload_type = match payload_type_id {
            1 => {
                let longitude_string = read_string(stream)?;
                let long = longitude_string.parse::<f64>().unwrap();

                let latitude_string = read_string(stream)?;

                let lat = latitude_string.parse::<f64>().unwrap();

                let location = Location::new(lat, long);
                let incident = Incident::new(location);

                PayloadTypes::IncidentLocation(IncidentPayload::new(incident))
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
    fn test_read_from() {
        let location = Location::new(1.0, 2.0);
        let incident = Incident::new(location.clone());
        let incident_payload = IncidentPayload::new(incident.clone());
        let payload = PayloadTypes::IncidentLocation(incident_payload.clone());

        let mut cursor = Cursor::new(Vec::new());
        payload.write_to(&mut cursor).unwrap();
        cursor.set_position(0);

        let read_payload = PayloadTypes::read_from(&mut cursor).unwrap();
        assert_eq!(read_payload, payload);
    }

    #[test]
    fn test_write_to() {
        let location = Location::new(1.0, 2.0);
        let incident = Incident::new(location.clone());
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
        let incident = Incident::new(location.clone());
        let incident_payload = IncidentPayload::new(incident.clone());
        let payload = PayloadTypes::IncidentLocation(incident_payload.clone());

        assert!(payload.as_any().is::<PayloadTypes>());
    }

    #[test]
    fn test_clone() {
        let location = Location::new(1.0, 2.0);
        let incident = Incident::new(location.clone());
        let incident_payload = IncidentPayload::new(incident.clone());
        let payload = PayloadTypes::IncidentLocation(incident_payload.clone());

        assert_eq!(payload.clone(), payload);
    }
}
