#[derive(Debug)]
#[allow(dead_code)]
pub struct Location {
    longitude: u16,
    latitude: u16,
}

impl Location {
    pub fn new(lat: String, long: String) -> Location {
        let latitude = lat.parse::<u16>().unwrap();
        let longitude = long.parse::<u16>().unwrap();

        Location {
            longitude,
            latitude,
        }
    }
}
