use crate::camera_system::camera::Camera;
pub struct CameraSystem {
    cameras: Vec<Camera>,
}

impl CameraSystem {
    pub fn new() -> CameraSystem {
        CameraSystem {
            cameras: Vec::new(),
        }
    }

    pub fn add_camera(&mut self, camera: Camera) {
        self.cameras.push(camera);
    }

    pub fn get_cameras(&self) -> &Vec<Camera> {
        &self.cameras
    }
}
