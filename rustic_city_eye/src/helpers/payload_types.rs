use std::{
    any::Any,
    io::{Error, ErrorKind},
};

use crate::{
    helpers::{incident_payload::IncidentPayload, location::Location},
    monitoring::incident::Incident,
    mqtt::{
        payload::Payload,
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
    fn write_to(&self, stream: &mut dyn std::io::prelude::Write) -> std::io::Result<()> {
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
                let latitude_string = read_string(stream)?;

                let location = Location::new(latitude_string, longitude_string);
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
