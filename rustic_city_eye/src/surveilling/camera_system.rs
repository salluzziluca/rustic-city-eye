use std::{
    collections::HashMap,
    sync::mpsc::{self, Sender},
};

use rand::Rng;

use crate::{
    mqtt::{
        client::Client, client_message, messages_config::MessagesConfig,
        protocol_error::ProtocolError, subscribe_config::SubscribeConfig,
        subscribe_properties::SubscribeProperties,
    },
    surveilling::camera::Camera,
    utils::location::Location,
};

use super::camera_error::CameraError;
#[derive(Debug)]
#[allow(dead_code)]
pub struct CameraSystem {
    send_to_client_channel: Sender<Box<dyn MessagesConfig + Send>>,
    camera_system_client: Client,
    cameras: HashMap<u32, Camera>,
}

impl CameraSystem {
    pub fn new(address: String) -> Result<CameraSystem, ProtocolError> {
        let connect_config =
            client_message::Connect::read_connect_config("./src/surveilling/connect_config.json")?;

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
            cameras: HashMap::new(),
        })
    }

    pub fn add_camera(&mut self, location: Location) -> Option<u32> {
        let mut rng = rand::thread_rng();

        let mut id = rng.gen();

        while self.cameras.contains_key(&id) {
            id = rng.gen();
        }

        let camera = Camera::new(location, id);
        self.cameras.insert(id, camera);
        Some(id)
    }

    pub fn get_cameras(&self) -> &HashMap<u32, Camera> {
        &self.cameras
    }

    pub fn get_camera_by_id(&mut self, id: u32) -> Option<&Camera> {
        let camera = self.cameras.get(&id);

        camera
    }

    pub fn get_camera(&mut self) -> Option<&Camera> {
        let keys: Vec<&u32> = self.cameras.keys().collect();
        if keys.is_empty() {
            None
        } else {
            let idx = rand::thread_rng().gen_range(0..keys.len());
            self.cameras.get(keys[idx])
        }
    }
    pub fn run_client(&mut self) -> Result<(), ProtocolError> {
        self.camera_system_client.client_run()?;
        Ok(())
    }

    pub fn send_message(
        &mut self,
        message: Box<dyn MessagesConfig + Send>,
    ) -> Result<(), CameraError> {
        let _ = match self.send_to_client_channel.send(message) {
            Ok(_) => {}
            Err(e) => {
                println!("Error sending message: {:?}", e);
                return Err(CameraError::SendError);
            }
        };
        Ok(())
    }
}

#[cfg(test)]

mod tests {
    use std::thread;

    use crate::mqtt::broker::Broker;
    use crate::mqtt::client_message::ClientMessage;

    use super::*;
    #[test]
    fn test01_new_camera_system_vacio() {
        let args = vec!["127.0.0.1".to_string(), "5000".to_string()];
        let addr = "127.0.0.1:5000";
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => {
                panic!("Error creating broker: {:?}", e)
            }
        };
        thread::spawn(move || {
            _ = broker.server_run();
        });

        thread::spawn(move || {
            let mut camera_system = CameraSystem::new(addr.to_string()).unwrap();
            assert_eq!(camera_system.get_cameras().len(), 0);
            assert_eq!(camera_system.get_camera_by_id(1), None);
            assert_eq!(camera_system.get_camera(), None);
        });
    }

    #[test]
    fn test02_add_camera() {
        let args = vec!["127.0.0.1".to_string(), "5000".to_string()];
        let addr = "127.0.0.1:5000";
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => {
                panic!("Error creating broker: {:?}", e)
            }
        };
        thread::spawn(move || {
            _ = broker.server_run();
        });

        thread::spawn(move || {
            let mut camera_system = CameraSystem::new(addr.to_string()).unwrap();
            let location = Location::new(1.0, 2.0);
            let id = camera_system.add_camera(location.clone()).unwrap();
            assert_eq!(camera_system.get_cameras().len(), 1);
            assert_eq!(
                camera_system.get_camera_by_id(id).unwrap().get_location(),
                location
            );
        });
    }

    #[test]
    fn test03_get_camera() {
        let args = vec!["127.0.0.1".to_string(), "5000".to_string()];
        let addr = "127.0.0.1:5000";
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => {
                panic!("Error creating broker: {:?}", e)
            }
        };
        thread::spawn(move || {
            _ = broker.server_run();
        });

        thread::spawn(move || {
            let mut camera_system = CameraSystem::new(addr.to_string()).unwrap();
            let location = Location::new(1.0, 2.0);
            let id = camera_system.add_camera(location.clone()).unwrap();
            assert_eq!(camera_system.get_cameras().len(), 1);
            assert_eq!(
                camera_system.get_camera_by_id(id).unwrap().get_location(),
                location
            );
            assert_eq!(camera_system.get_camera().unwrap().get_location(), location);
        });
    }

    #[test]
    fn test04_run_client() {
        let args = vec!["127.0.0.1".to_string(), "5000".to_string()];
        let addr = "127.0.0.1:5000";
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => {
                panic!("Error creating broker: {:?}", e)
            }
        };
        thread::spawn(move || {
            _ = broker.server_run();
        });

        thread::spawn(move || {
            let mut camera_system = CameraSystem::new(addr.to_string()).unwrap();
            assert_eq!(camera_system.run_client().is_ok(), true);
        });
    }

    #[test]

    fn test05_envio_de_mensaje() {
        let args = vec!["127.0.0.1".to_string(), "5000".to_string()];
        let addr = "127.0.0.1:5000";
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => {
                panic!("Error creating broker: {:?}", e)
            }
        };
        thread::spawn(move || {
            _ = broker.server_run();
        });

        thread::spawn(move || {
            let mut camera_system = CameraSystem::new(addr.to_string()).unwrap();

            let pingreq = ClientMessage::Pingreq;

            match camera_system.send_message(Box::new(pingreq)) {
                Ok(_) => {}
                Err(e) => {
                    panic!("Error sending message: {:?}", e);
                }
            }
        });
    }
}
