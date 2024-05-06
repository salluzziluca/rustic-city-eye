use std::io::{BufRead, BufReader, Error, Read};

use crate::{
    camera_system::camera::Camera,
    mqtt::{client::Client, connect_properties, protocol_error::ProtocolError, will_properties},
};
pub struct CameraSystem {
    args: Vec<String>,
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
            args.clone(),
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
            args,
            camera_system_client,
            cameras,
        })
    }

    pub fn app_run(&mut self, stream: &mut dyn Read) -> Result<(), Error> {
        let reader = BufReader::new(stream);

        for line in reader.lines() {
            if let Ok(line) = line {
                if line.starts_with("publish:") {
                    let (_, post_colon) = line.split_at(8); // "publish:" is 8 characters
                    let message = post_colon.trim(); // remove leading/trailing whitespace
                    println!("Publishing message: {}", message);
                    self.camera_system_client.publish_message(message);
                } else if line.starts_with("subscribe:") {
                    let (_, post_colon) = line.split_at(10); // "subscribe:" is 10 characters
                    let topic = post_colon.trim(); // remove leading/trailing whitespace
                    println!("Subscribing to topic: {}", topic);
                    self.camera_system_client.subscribe(topic);
                } else {
                    println!("Comando no reconocido: {}", line);
                }
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Error al leer linea",
                ));
            }
        }
        Ok(())
    }

    pub fn add_camera(&mut self) -> Result<(), ProtocolError> {
        let camera = Camera::new(self.args.clone())?;

        self.cameras.push(camera);

        Ok(())
    }

    pub fn get_cameras(&self) -> &Vec<Camera> {
        &self.cameras
    }
}
