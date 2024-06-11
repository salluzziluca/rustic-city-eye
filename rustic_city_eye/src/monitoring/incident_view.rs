use walkers::Position;

use crate::ImagesPluginData;

pub struct IncidentView {
    pub image: ImagesPluginData,
    pub position: Position,
    pub clicked: bool,
}

impl IncidentView {
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
}
