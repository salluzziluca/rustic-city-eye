use walkers::Position;

use crate::ImagesPluginData;

pub struct CameraView {
    pub image: ImagesPluginData,
    pub position: Position,
    pub radius: ImagesPluginData,
}
