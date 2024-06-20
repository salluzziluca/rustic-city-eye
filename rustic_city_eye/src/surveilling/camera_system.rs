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
    utils::payload_types::PayloadTypes,
};

const AREA_DE_ALCANCE: f64 = 10.0;

use super::camera_error::CameraError;
#[derive(Debug)]
#[allow(dead_code)]
pub struct CameraSystem {
    send_to_client_channel: Sender<Box<dyn MessagesConfig + Send>>,
    camera_system_client: Client,
    cameras: HashMap<u32, Camera>,
    reciev_from_client: mpsc::Receiver<client_message::ClientMessage>,
}

impl CameraSystem {
    pub fn new(address: String) -> Result<CameraSystem, ProtocolError> {
        let connect_config =
            client_message::Connect::read_connect_config("./src/surveilling/connect_config.json")?;

        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();
        let camera_system_client = match Client::new(rx, address, connect_config, tx2) {
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
            reciev_from_client: rx2,
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

        let reciever = self.reciev_from_client.recv().unwrap();
        match reciever {
            client_message::ClientMessage::Publish {
                topic_name: _,
                payload,
                properties: _,
                packet_id: _,
                qos: _,
                retain_flag: _,
                dup_flag: _,
            } => {
                if let PayloadTypes::IncidentLocation(payload) = payload {
                    let location = payload.get_incident().get_location();
                    let _ = self.activate_cameras(location);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Recibe una location y activas todas las camaras que esten a menos de AREA_DE_ALCANCE de esta.
    ///
    /// Al activaralas se las pasa de modo ahorro de energia a modo activo
    pub fn activate_cameras(&mut self, location: Location) -> Result<(), CameraError> {
        for camera in self.cameras.values_mut() {
            let distancia = camera.get_location().distance(location.clone());
            if distancia < AREA_DE_ALCANCE {
                camera.set_sleep_mode(false);
            }
        }

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
    use std::sync::{Arc, Condvar, Mutex};
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

    #[test]
    fn test06_activar_camara() {
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
            let incident_location = Location::new(1.0, 2.0);
            camera_system.activate_cameras(incident_location).unwrap();
            for camera in camera_system.get_cameras().values() {
                assert_eq!(camera.get_sleep_mode(), false);
            }
        });
    }

    #[test]
    fn test07_activar_multiples_camaras() {
        let args = vec!["127.0.0.1".to_string(), "5005".to_string()];
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => panic!("Error creating broker: {:?}", e),
        };

        let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
        let server_ready_clone = server_ready.clone();
        thread::spawn(move || {
            {
                let (lock, cvar) = &*server_ready_clone;
                let mut ready = lock.lock().unwrap();
                *ready = true;
                cvar.notify_all();
            }
            let _ = broker.server_run();
        });

        // Wait for the server to start
        {
            let (lock, cvar) = &*server_ready;
            let mut ready = lock.lock().unwrap();
            while !*ready {
                ready = cvar.wait(ready).unwrap();
            }
        }
        let addr = "127.0.0.1:5005";
        let handle = thread::spawn(move || {
            let mut camera_system = CameraSystem::new(addr.to_string()).unwrap();
            let location = Location::new(1.0, 20.0);
            let id = camera_system.add_camera(location.clone()).unwrap();
            let location2 = Location::new(1.0, 2.0);
            let id2 = camera_system.add_camera(location2.clone()).unwrap();
            let location3 = Location::new(10.0, 2.0);
            let id3 = camera_system.add_camera(location3.clone()).unwrap();
            let location4 = Location::new(10.0, 20.0);
            let id4 = camera_system.add_camera(location4.clone()).unwrap();
            let location5 = Location::new(1.0, 2.0);
            let id5 = camera_system.add_camera(location5.clone()).unwrap();
            assert_eq!(camera_system.get_cameras().len(), 5);
            assert_eq!(
                camera_system.get_camera_by_id(id).unwrap().get_location(),
                location
            );
            let incident_location = Location::new(1.0, 2.0);
            camera_system.activate_cameras(incident_location).unwrap();
            assert_eq!(
                camera_system.get_camera_by_id(id).unwrap().get_sleep_mode(),
                true
            );
            assert_eq!(
                camera_system
                    .get_camera_by_id(id2)
                    .unwrap()
                    .get_sleep_mode(),
                false
            );
            assert_eq!(
                camera_system
                    .get_camera_by_id(id3)
                    .unwrap()
                    .get_sleep_mode(),
                false
            );
            assert_eq!(
                camera_system
                    .get_camera_by_id(id4)
                    .unwrap()
                    .get_sleep_mode(),
                true
            );
            assert_eq!(
                camera_system
                    .get_camera_by_id(id5)
                    .unwrap()
                    .get_sleep_mode(),
                false
            );
        });
        match handle.join() {
            Ok(_) => {}
            Err(e) => {
                panic!("Error joining thread: {:?}", e);
            }
        }
    }
}
