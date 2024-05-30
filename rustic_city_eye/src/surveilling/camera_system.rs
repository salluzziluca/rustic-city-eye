use std::sync::mpsc::{self, Sender};

use crate::{
    helpers::location::Location,
    mqtt::{
        client::Client, connect_config::ConnectConfig, connect_properties,
        messages_config::MessagesConfig, protocol_error::ProtocolError,
        subscribe_config::SubscribeConfig, subscribe_properties::SubscribeProperties,
        will_properties,
    },
    surveilling::camera::Camera,
};
#[derive(Debug)]
#[allow(dead_code)]
pub struct CameraSystem {
    send_to_client_channel: Sender<Box<dyn MessagesConfig + Send>>,
    camera_system_client: Client,
    cameras: Vec<Camera>,
}

impl CameraSystem {
    pub fn new(args: Vec<String>) -> Result<CameraSystem, ProtocolError> {
        let address = args[0].clone() + ":" + &args[1];
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

        let connect_config = ConnectConfig::new(
            true,
            true,
            1,
            true,
            35,
            connect_properties,
            "juancito".to_string(),
            will_properties,
            "camera system".to_string(),
            "soy el monitoring y me desconecte".to_string(),
            args[2].clone(),
            args[3].clone(),
        );

        let (tx, rx) = mpsc::channel();

        let camera_system_client = match Client::new(rx, address, connect_config) {
            Ok(client) => client,
            Err(err) => return Err(err),
        };

        let subscribe_config = SubscribeConfig::new(
            "incidente".to_string(),
            SubscribeProperties::new(
                0,
                vec![("propiedad".to_string(), "valor".to_string())],
                vec![0, 1, 2, 3],
            ),
        );

        let _ = tx.send(Box::new(subscribe_config));

        Ok(CameraSystem {
            send_to_client_channel: tx,
            camera_system_client,
            cameras: Vec::new(),
        })
    }

    pub fn add_camera(&mut self, location: Location) {
        let camera = Camera::new(location);
        self.cameras.push(camera);
    }

    pub fn get_cameras(&self) -> &Vec<Camera> {
        &self.cameras
    }

    pub fn run_client(&mut self) -> Result<(), ProtocolError> {
        self.camera_system_client.client_run()?;
        Ok(())
    }
}
