/// Contiene una localizacion especifica en el mapa.
///
/// La idea es que implemente el trait de Payload que nos provee la API del cliente,
/// de forma tal que al ingresar un incidente en la aplicacion, se envie un packet
/// del tipo Publish con la localizacion del incidente como Payload,
/// para que las distintas unidades de la aplicacion sepan donde se encuentran
/// los incidentes a resolver.
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub long: f64,
    pub lat: f64,
}

impl Location {
    pub fn new(lat: f64, long: f64) -> Location {
        Location {
            lat,
            long,
        }
    }

    // pub fn parse_to_string(&self) -> String {
    //     format!("({}, {})", self.latitude, self.longitude)
    // }
}
