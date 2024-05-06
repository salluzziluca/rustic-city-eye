use crate::{
    camera_system::camera::Camera,
    mqtt::{client::Client, connect_properties, protocol_error::ProtocolError, will_properties},
};
pub struct CameraSystem {
    camera_system_client: Client,
    cameras: Vec<Camera>,
}

impl CameraSystem {
    pub fn new(args: Vec<String>) -> Result<CameraSystem, ProtocolError> {
        let cameras = Vec::new();

        let will_properties = will_properties::WillProperties::new(
            1,
            1,
            1,
            "a".to_string(),
            "a".to_string(),
            [1, 2, 3].to_vec(),
            vec![("a".to_string(), "a".to_string())],
        );

        let connect_properties = connect_properties::ConnectProperties::new(
            30,
            1,
            20,
            20,
            true,
            true,
            vec![("hola".to_string(), "chau".to_string())],
            "auth".to_string(),
            vec![1, 2, 3],
        );

        let camera_system_client = match Client::new(
            args,
            will_properties,
            connect_properties,
            true,
            true,
            1,
            true,
            "prueba".to_string(),
            "".to_string(),
            35,
            "kvtr33".to_string(),
            "camera_system".to_string(),
            "soy el camera_system y me desconecté".to_string(),
        ) {
            Ok(client) => client,
            Err(err) => return Err(err),
        };

        Ok(CameraSystem {
            camera_system_client,
            cameras,
        })
    }

    pub fn add_camera(&mut self, camera: Camera) {
        self.cameras.push(camera);
    }

    pub fn get_cameras(&self) -> &Vec<Camera> {
        &self.cameras
    }
}
