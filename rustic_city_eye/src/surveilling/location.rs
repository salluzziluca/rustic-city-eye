#[derive(Debug, Clone)]
#[allow(dead_code)]
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
