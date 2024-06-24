use crate::utils::location::Location;

#[derive(Debug,Clone)]
pub struct DroneLocation {
    pub id: u32,
    pub location: Location,
}