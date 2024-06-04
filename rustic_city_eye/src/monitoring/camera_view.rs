use walkers::Position;

use crate::plugins::ImagesPluginData;

pub struct CameraView {
    pub image: ImagesPluginData,
    pub position: Position,
    pub radius: ImagesPluginData,
}
