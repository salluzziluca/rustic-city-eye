use std::sync::mpsc::{self, Sender};

use crate::{
    mqtt::{client::Client, connect_properties, protocol_error::ProtocolError, will_properties},
    surveilling::{camera::Camera, location::Location},
};
#[derive(Debug)]
#[allow(dead_code)]
pub struct CameraSystem {
    // args: Vec<String>,
    send_to_client_channel: Sender<String>,
    camera_system_client: Client,
    cameras: Vec<Camera>,
}

impl CameraSystem {
    pub fn new(args: Vec<String>) -> Result<CameraSystem, ProtocolError> {
        // if args.len() != CLIENT_ARGS {
        //     let app_name = &args[0];
        //     println!("Usage:\n{:?} <host> <puerto>", app_name);
        //     return Err(ProtocolError::InvalidNumberOfArguments);
        // }

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

        let (tx, rx) = mpsc::channel();

        let camera_system_client = match Client::new(
            rx,
            address,
            will_properties,
            connect_properties,
            true,
            true,
            1,
            true,
            args[2].clone(),
            args[3].clone(),
            35,
            "kvtr33".to_string(),
            "camera_system".to_string(),
            "soy el camera_system y me desconecté".to_string(),
        ) {
            Ok(client) => client,
            Err(err) => return Err(err),
        };

        Ok(CameraSystem {
            send_to_client_channel: tx,
            // args,
            camera_system_client,
            cameras: Vec::new(),
        })
    }

    pub fn add_camera(&mut self, location: Location) {
        let camera = Camera::new(location);
        self.cameras.push(camera);
        println!("mis camaritas: {:?}", self.cameras);

        let sub_msg = "subscribe: accidente".to_string();
        let _ = self.send_to_client_channel.send(sub_msg);
    }

    pub fn get_cameras(&self) -> &Vec<Camera> {
        &self.cameras
    }

    pub fn run_client(&mut self) -> Result<(), ProtocolError> {
        self.camera_system_client.client_run()?;
        Ok(())
    }
}
