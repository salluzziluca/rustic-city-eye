use std::{
    any::Any,
    io::{Error, ErrorKind},
};

use crate::mqtt::{
    payload::Payload,
    reader::{read_string, read_u8},
    writer::{write_string, write_u8},
};

#[derive(Clone, Debug, PartialEq)]
pub enum PayloadTypes {
    IncidentLocation {
        id: u8,
        longitude: f64,
        latitude: f64,
    },
}

impl Payload for PayloadTypes {
    fn write_to(&self, stream: &mut dyn std::io::prelude::Write) -> std::io::Result<()> {
        match self {
            PayloadTypes::IncidentLocation {
                id,
                longitude,
                latitude,
            } => {
                write_u8(stream, id)?;

                let longitude_string = longitude.to_string();
                write_string(stream, &longitude_string)?;

                let latitude_string = latitude.to_string();
                write_string(stream, &latitude_string)?;

                Ok(())
            }
        }
    }

    // fn read_from(&self, stream: &mut dyn std::io::prelude::Read) -> Result<PayloadTypes, std::io::Error> {
    //     let payload_type_id = read_u8(stream)?;

    //     let payload_type = match payload_type_id {
    //         1 => {
    //             let longitude_string = read_string(stream)?;
    //             let longitude: f64 = longitude_string.parse().map_err(|e| {
    //                 Error::new(
    //                     ErrorKind::InvalidData,
    //                     format!("Error al parsear la longitud: {}", e),
    //                 )
    //             })?;

    //             let latitude_string = read_string(stream)?;
    //             let latitude: f64 = latitude_string.parse().map_err(|e| {
    //                 Error::new(
    //                     ErrorKind::InvalidData,
    //                     format!("Error al parsear la longitud: {}", e),
    //                 )
    //             })?;

    //             PayloadTypes::IncidentLocation { id: payload_type_id, longitude, latitude }
    //         },
    //         _ => {
    //             return Err(Error::new(
    //                 ErrorKind::InvalidData,
    //                 format!("Error while reading payload"),
    //             ))
    //         }
    //     };
    //     Ok(payload_type)
    // }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PayloadTypes {
    pub fn read_from(
        stream: &mut dyn std::io::prelude::Read,
    ) -> Result<PayloadTypes, std::io::Error> {
        let payload_type_id = read_u8(stream)?;

        let payload_type = match payload_type_id {
            1 => {
                let longitude_string = read_string(stream)?;
                let longitude: f64 = longitude_string.parse().map_err(|e| {
                    Error::new(
                        ErrorKind::InvalidData,
                        format!("Error al parsear la longitud: {}", e),
                    )
                })?;

                let latitude_string = read_string(stream)?;
                let latitude: f64 = latitude_string.parse().map_err(|e| {
                    Error::new(
                        ErrorKind::InvalidData,
                        format!("Error al parsear la longitud: {}", e),
                    )
                })?;

                PayloadTypes::IncidentLocation {
                    id: payload_type_id,
                    longitude,
                    latitude,
                }
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Error while reading payload"),
                ))
            }
        };
        Ok(payload_type)
    }
}
