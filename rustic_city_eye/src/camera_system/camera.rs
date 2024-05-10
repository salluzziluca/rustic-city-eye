use std::io::{Error, Read};

use crate::mqtt::client::Client;
use crate::mqtt::protocol_error::ProtocolError;
use crate::mqtt::{connect_properties, will_properties};

pub struct Camera {
    camera_client: Client,
}

impl Camera {
    pub fn new(args: Vec<String>) -> Result<Camera, ProtocolError> {
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

        let camera_client = match Client::new(
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
            "camara".to_string(),
            "soy una camara y me desconecté".to_string(),
        ) {
            Ok(client) => client,
            Err(err) => return Err(err),
        };

        Ok(Camera { camera_client })
    }

    pub fn camera_run(&mut self, stream: Box<dyn Read + Send>) -> Result<(), Error> {
        // let reader = BufReader::new(stream);

        // for line in reader.lines() {
        //     if let Ok(line) = line {
        //         if line.starts_with("publish:") {
        //             let (_, post_colon) = line.split_at(8); // "publish:" is 8 characters
        //             let message = post_colon.trim(); // remove leading/trailing whitespace
        //             println!("Publishing message: {}", message);
        //             self.camera_client.publish_message(message);
        //         } else if line.starts_with("subscribe:") {
        //             let (_, post_colon) = line.split_at(10); // "subscribe:" is 10 characters
        //             let topic = post_colon.trim(); // remove leading/trailing whitespace
        //             println!("Subscribing to topic: {}", topic);
        //             self.camera_client.subscribe(topic);
        //         } else {
        //             println!("Comando no reconocido: {}", line);
        //         }
        //     } else {
        //         return Err(std::io::Error::new(
        //             std::io::ErrorKind::Other,
        //             "Error al leer linea",
        //         ));
        //     }
        // }
        self.camera_client.client_run(Box::new(stream));
        Ok(())
    }
}
