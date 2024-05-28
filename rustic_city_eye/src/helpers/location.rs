// use std::{
//     any::Any,
//     io::{Error, ErrorKind, Read, Write},
// };

// use crate::mqtt::{payload::Payload, reader::read_string, writer::write_string};

/// Contiene una localizacion especifica en el mapa.
///
/// La idea es que implemente el trait de Payload que nos provee la API del cliente,
/// de forma tal que al ingresar un incidente en la aplicacion, se envie un packet
/// del tipo Publish con la localizacion del incidente como Payload,
/// para que las distintas unidades de la aplicacion sepan donde se encuentran
/// los incidentes a resolver.
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub longitude: f64,
    pub latitude: f64,
}

/// Implemento el trait Payload para poder enviarlo en nuestros packets del tipo
/// Publish.
// impl Payload for Location {
//     fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()> {
//         let longitude_string = self.longitude.to_string();
//         write_string(stream, &longitude_string)?;

//         let latitude_string = self.latitude.to_string();
//         write_string(stream, &latitude_string)?;

//         Ok(())
//     }

//     fn read_from(&self, stream: &mut dyn Read) -> Result<Box<dyn Payload>, Error> {
//         let longitude_string = read_string(stream)?;
//         let longitude: f64 = longitude_string.parse().map_err(|e| {
//             Error::new(
//                 ErrorKind::InvalidData,
//                 format!("Error al parsear la longitud: {}", e),
//             )
//         })?;

//         let latitude_string = read_string(stream)?;
//         let latitude: f64 = latitude_string.parse().map_err(|e| {
//             Error::new(
//                 ErrorKind::InvalidData,
//                 format!("Error al parsear la longitud: {}", e),
//             )
//         })?;

//         let location = Location {
//             longitude,
//             latitude,
//         };

//         Ok(Box::new(location))
//     }

//     fn as_any(&self) -> &dyn Any {
//         self
//     }
// }

impl Location {
    pub fn new(lat: String, long: String) -> Location {
        let latitude = lat.parse::<f64>().unwrap();
        let longitude = long.parse::<f64>().unwrap();

        Location {
            longitude,
            latitude,
        }
    }

    pub fn parse_to_string(&self) -> String {
        format!("({}, {})", self.latitude, self.longitude)
    }
}

// #[cfg(test)]
// mod tests {
//     use std::io::Cursor;

//     use super::*;

//     #[test]
//     fn test_01_writing_a_location_payload_ok() -> std::io::Result<()> {
//         let location = Location::new("1.4".to_string(), "12.9".to_string());

//         let mut cursor = Cursor::new(Vec::<u8>::new());
//         location.write_to(&mut cursor)?;

//         cursor.set_position(0);

//         let location_readed = location.read_from(&mut cursor)?;

//         if location_readed.is::<Location>() {
//             let location = location_readed.downcast_ref::<Location>().unwrap();
//             assert_eq!(
//                 *location,
//                 Location::new("1.4".to_string(), "12.9".to_string())
//             );
//         } else {
//             println!("No es una instancia de Location");
//         }

//         Ok(())
//     }
// }
