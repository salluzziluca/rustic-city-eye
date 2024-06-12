use walkers::Position;

use crate::plugins::ImagesPluginData;

pub struct DroneView {
    pub image: ImagesPluginData,
    pub position: Position,
    pub radius: ImagesPluginData,
}