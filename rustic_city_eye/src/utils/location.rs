use serde::Deserialize;
/// Contiene una localizacion especifica en el mapa.
///
/// La idea es que implemente el trait de Payload que nos provee la API del cliente,
/// de forma tal que al ingresar un incidente en la aplicacion, se envie un packet
/// del tipo Publish con la localizacion del incidente como Payload,
/// para que las distintas unidades de la aplicacion sepan donde se encuentran
/// los incidentes a resolver.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Location {
    pub long: f64,
    pub lat: f64,
}

impl Location {
    pub fn new(lat: f64, long: f64) -> Location {
        Location { lat, long }
    }

    pub fn get_latitude(&self) -> f64 {
        self.lat
    }

    pub fn get_longitude(&self) -> f64 {
        self.long
    }

    /// Recive una ubicacion y calcula la distancia euclideana entre esta y la ubicacion de la propia location
    pub fn distance(&self, location: Location) -> f64 {
        let distancia = ((location.get_latitude() - self.get_latitude()).powi(2)
            + (location.get_longitude() - self.get_longitude()).powi(2))
        .sqrt();
        distancia
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_new() {
        let location = Location::new(1.0, 2.0);
        assert_eq!(location.lat, 1.0);
        assert_eq!(location.long, 2.0);
    }

    #[test]
    fn test_distance() {
        let location = Location::new(1.0, 2.0);
        let location2 = Location::new(2.0, 3.0);
        assert_eq!(location.distance(location2), 1.4142135623730951);
    }
}
