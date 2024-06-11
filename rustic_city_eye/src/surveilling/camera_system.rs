use std::sync::mpsc::{self, Sender};

use crate::{
    mqtt::{
        client::Client, connect::connect_config::ConnectConfig, messages_config::MessagesConfig,
        protocol_error::ProtocolError, subscribe_config::SubscribeConfig,
        subscribe_properties::SubscribeProperties,
    },
    surveilling::camera::Camera,
    utils::location::Location,
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
        let connect_config =
            ConnectConfig::read_connect_config("./src/surveilling/connect_config.json")?;

        let address = args[0].clone() + ":" + &args[1];

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
