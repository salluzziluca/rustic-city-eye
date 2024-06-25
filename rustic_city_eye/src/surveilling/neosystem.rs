use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use crate::{
    mqtt::{
        client::{Client, ClientTrait},
        client_message::{ClientMessage, Connect},
        messages_config::MessagesConfig,
        protocol_error::ProtocolError,
        subscribe_config::SubscribeConfig,
        subscribe_properties::SubscribeProperties,
    },
    utils::location::Location,
};

use super::neocamera::Camera;

/// Tendrá la ubicación y el estado de cada cámara y permitirá agregar,
/// quitar y modificar las mismas.
#[derive(Debug)]
pub struct CameraSystem {
    cameras: Vec<Camera>,
    camera_system_client: Client,
    send_to_client_channel: Sender<Box<dyn MessagesConfig + Send>>,
    receive_from_client_channel: Arc<Mutex<Receiver<ClientMessage>>>,
}

impl CameraSystem {
    pub fn new(address: String) -> Result<CameraSystem, ProtocolError> {
        let (tx_1, rx_1) = mpsc::channel();
        let (tx_2, rx_2) = mpsc::channel();

        let connect = Connect::read_connect_config("./src/surveilling/connect_config.json")?;

        let camera_system_client = Client::new(rx_2, address, connect, tx_1)?;

        let system = CameraSystem {
            cameras: Vec::new(),
            camera_system_client,
            send_to_client_channel: tx_2,
            receive_from_client_channel: Arc::new(Mutex::new(rx_1)),
        };

        system.subscribe_to_topics()?;

        Ok(system)
    }


    /// Se suscribe al topic incidente para poder recibir los incidentes
    /// que la monitoring app envie.
    fn subscribe_to_topics(&self) -> Result<(), ProtocolError> {
        let client_id = self.camera_system_client.get_client_id();
        let subscribe_properties: SubscribeProperties = SubscribeProperties::new(1, Vec::new());
        let subscribe_config = SubscribeConfig::new(
            "incidente".to_string(),
            1,
            subscribe_properties,
            client_id.clone(),
        );
        match self.send_to_client_channel.send(Box::new(subscribe_config)) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Error sending message: {:?}", e);
                return Err(ProtocolError::SubscribeError);
            }
        }
    }

    /// Corre el cliente de la aplicacion, ademas de quedarse esperando por mensajes que le
    /// envie su cliente.
    /// 
    /// Si llega un publish, se leera su payload y se pondra en marcha el proceso de activacion de camaras
    /// si es necesario.
    pub fn run_client(&mut self) -> Result<(), ProtocolError> {
        self.camera_system_client.client_run()?;

        let receiver_from_client_clone = Arc::clone(&self.receive_from_client_channel);

        thread::spawn(move || loop {
            let lock = receiver_from_client_clone.lock().unwrap();
            if let Ok(message) = lock.try_recv() {
                match message {
                    ClientMessage::Publish {
                        packet_id: _,
                        topic_name: _,
                        qos: _,
                        retain_flag: _,
                        payload,
                        dup_flag: _,
                        properties: _,
                    } => {
                        println!("payload {:?}", payload);
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }


    /// Agrega una camara en una location determinada.
    pub fn add_camera(&mut self, location: Location) {
        let camera_id = self.cameras.len();

        let new_camera = Camera::new(location, camera_id, 10.0);

        self.cameras.push(new_camera);
    }

    /// Se toma la suposicion de que dos camaras no van a tener la misma location.
    /// Elimina la camara colocada en la location indicada(si existe).
    pub fn delete_camera(&mut self, location: Location) {
        if let Some(index) = self
            .cameras
            .iter()
            .position(|camera| camera.is_at_location(location.clone()))
        {
            self.cameras.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use crate::mqtt::broker::Broker;

    use super::*;

    #[test]
    fn test_01_system_creation_ok() {
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

        let camera_system = CameraSystem::new(addr.to_string());
        assert!(camera_system.is_ok());
    }

    #[test]
    fn test_02_adding_camera_to_system_ok() {
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

        let mut camera_system = CameraSystem::new(addr.to_string()).unwrap();
        let camera_location = Location::new(1.2, 2.1);

        camera_system.add_camera(camera_location);
    }

    #[test]
    fn test_03_adding_more_cameras_to_system_ok() {
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

        let mut camera_system = CameraSystem::new(addr.to_string()).unwrap();
        let camera_location_one = Location::new(1.2, 2.1);
        let camera_location_two = Location::new(3.4, 4.3);
        let camera_location_three = Location::new(5.6, 6.5);

        camera_system.add_camera(camera_location_one);
        camera_system.add_camera(camera_location_two);
        camera_system.add_camera(camera_location_three);

        println!("system {:?}", camera_system);
    }

    #[test]
    fn test_04_delete_cameras_ok() {
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

        let mut camera_system = CameraSystem::new(addr.to_string()).unwrap();
        let camera_location_one = Location::new(1.2, 2.1);
        let camera_location_two = Location::new(3.4, 4.3);
        let camera_location_three = Location::new(5.6, 6.5);

        camera_system.add_camera(camera_location_one.clone());
        camera_system.add_camera(camera_location_two);
        camera_system.add_camera(camera_location_three);

        camera_system.delete_camera(camera_location_one);

        println!("system {:?}", camera_system);
    }
}
