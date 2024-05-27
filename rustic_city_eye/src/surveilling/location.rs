#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Location {
    pub longitude: f64,
    pub latitude: f64,
}

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
