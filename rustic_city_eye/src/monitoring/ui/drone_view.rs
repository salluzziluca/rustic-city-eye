use rustic_city_eye::drones::drone::DRONE_SPEED;
use walkers::Position;

use crate::ImagesPluginData;

pub struct DroneView {
    pub image: ImagesPluginData,
    pub position: Position,
    pub target_position: Position,
    pub clicked: bool,
    // pub id: u32,
}

impl DroneView {
    pub fn select(&mut self, position: Option<Position>) -> bool {
        if let Some(position) = position {
            self.clicked = self.distance(position) < 0.001;
        }
        self.clicked
    }

    pub fn distance(&self, position: Position) -> f32 {
        let dist_lat = self.position.lat() - position.lat();
        let dist_lon = self.position.lon() - position.lon();
        (dist_lat * dist_lat + dist_lon * dist_lon).sqrt() as f32
    }

    pub fn move_towards(&mut self) {
        // println!("Target: {:?}", self.target_position);
        let dist_lat = self.position.lat() - self.target_position.lat();
        let dist_lon = self.position.lon() - self.target_position.lon();
        let dist = (dist_lat * dist_lat + dist_lon * dist_lon).sqrt();
        if dist > DRONE_SPEED * 0.05{
            let lat = self.position.lat() - dist_lat/dist * DRONE_SPEED*0.05;
            let lon = self.position.lon() - dist_lon/dist * DRONE_SPEED*0.05;
            self.position = Position::from_lat_lon(lat, lon);
        } else {
            self.position = self.target_position;
        }
    }
}
