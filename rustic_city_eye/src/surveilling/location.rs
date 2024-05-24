#[derive(Debug)]
#[allow(dead_code)]
pub struct Location {
    longitude: f64,
    latitude: f64,
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
}
