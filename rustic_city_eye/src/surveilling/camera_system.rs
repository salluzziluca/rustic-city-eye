use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
};

use rand::Rng;

use crate::{
    mqtt::{
        client::{Client, ClientTrait},
        client_message::{self, ClientMessage},
        messages_config::MessagesConfig,
        protocol_error::ProtocolError,
        subscribe_config::SubscribeConfig,
        subscribe_properties::SubscribeProperties,
    },
    surveilling::camera::Camera,
    utils::{location::Location, payload_types::PayloadTypes},
};

const AREA_DE_ALCANCE: f64 = 10.0;
const NIVEL_DE_PROXIMIDAD_MAXIMO: f64 = 1.0;

use super::camera_error::CameraError;
#[derive(Debug)]
#[allow(dead_code)]
pub struct CameraSystem<T: ClientTrait + Clone> {
    pub send_to_client_channel: Arc<Mutex<Sender<Box<dyn MessagesConfig + Send>>>>,
    camera_system_client: T,
    cameras: HashMap<u32, Camera>,
    reciev_from_client: Arc<Mutex<Receiver<client_message::ClientMessage>>>,
}

impl<T: ClientTrait + Clone> Clone for CameraSystem<T> {
    fn clone(&self) -> Self {
        CameraSystem {
            send_to_client_channel: Arc::clone(&self.send_to_client_channel),
            camera_system_client: self.camera_system_client.clone(),
            cameras: self.cameras.clone(),
            reciev_from_client: Arc::clone(&self.reciev_from_client),
        }
    }
}

impl<T: ClientTrait + Clone> CameraSystem<T> {
    /// Crea un nuevo camera system con un cliente de mqtt
    ///
    /// Envia un connect segun la configuracion del archivo connect_config.json
    ///
    /// Se subscribe a los mensajes de tipo "accidente"
    ///
    pub fn new<F>(address: String, client_factory: F) -> Result<CameraSystem<T>, ProtocolError>
    where
        F: FnOnce(
            mpsc::Receiver<Box<dyn MessagesConfig + Send>>,
            String,
            client_message::Connect,
            Sender<client_message::ClientMessage>,
        ) -> Result<T, ProtocolError>,
    {
        let connect_config =
            client_message::Connect::read_connect_config("./src/surveilling/connect_config.json")?;

        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();
        let camera_system_client = client_factory(rx, address, connect_config, tx2)?;

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
            send_to_client_channel: Arc::new(Mutex::new(tx)),
            camera_system_client,
            cameras: HashMap::new(),
            reciev_from_client: Arc::new(Mutex::new(rx2)),
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

    /// recibe los diferentes mensajes reenviados por el cliente.
    ///
    /// Si el mensaje es un publish con topic accidente, activa las camaras cercanas a la location del incidente.
    ///
    /// Si el mensaje es un publish con topic accidenteresuelto, desactiva las camaras cercanas a la location del incidente.
    ///
    /// Recibe un reciever opcional para poder testear la funcion, si este es None, utiliza el propio del broker
    pub fn run_client(
        &mut self,
        reciever: Option<Receiver<ClientMessage>>,
    ) -> Result<(), ProtocolError> {
        self.camera_system_client.client_run()?;

        loop {
            let lock = self.reciev_from_client.lock().unwrap();
            if let Some(ref reciever) = reciever {
                match reciever.try_recv() {
                    Ok(client_message::ClientMessage::Publish {
                        topic_name,
                        payload: PayloadTypes::IncidentLocation(payload),
                        ..
                    }) => {
                        println!("topic_name: {:?}", topic_name);
                        if topic_name != "accidente" {
                            continue;
                        }
                        let location = payload.get_incident().get_location();
                        drop(lock); // Release the lock here
                        self.activate_cameras(location)
                            .map_err(|e| ProtocolError::CameraError(e.to_string()))?;

                        continue;
                    }
                    Ok(_) => continue, // Handle other message types if necessary
                    Err(e) => {
                        return Err(ProtocolError::ChanellError(e.to_string()));
                    }
                }
            } else {
                match lock.try_recv() {
                    Ok(client_message::ClientMessage::Publish {
                        topic_name,
                        payload: PayloadTypes::IncidentLocation(payload),
                        ..
                    }) => {
                        if topic_name == "accidente" {
                            let location = payload.get_incident().get_location();
                            drop(lock); // Release the lock here
                            self.activate_cameras(location)
                                .map_err(|e| ProtocolError::CameraError(e.to_string()))?;

                            continue;
                        } else if topic_name == "accidenteresuelto" {
                            let location = payload.get_incident().get_location();
                            drop(lock); // Release the lock here
                            self.deactivate_cameras(location)
                                .map_err(|e| ProtocolError::CameraError(e.to_string()))?;
                            continue;
                        }
                    }
                    Ok(_) => continue, // Handle other message types if necessary
                    Err(e) => {
                        return Err(ProtocolError::ChanellError(e.to_string()));
                    }
                }
            }
        }
    }

    /// Recibe una location y activas todas las camaras que esten a menos de AREA_DE_ALCANCE de esta.
    ///
    /// Al activaralas se las pasa de modo ahorro de energia a modo activo
    ///
    /// Adicionalmente, cada camara activada despues avisa a las camaras colindantes para que,
    /// si estan a la distancia requerida, tambien se activen.
    pub fn activate_cameras(&mut self, location: Location) -> Result<(), CameraError> {
        // Collect the locations that need to be activated first
        let locations_to_activate: Vec<Location> = self
            .cameras
            .values_mut()
            .filter_map(|camera| {
                let distancia = camera.get_location().distance(location.clone());
                if distancia <= AREA_DE_ALCANCE {
                    camera.set_sleep_mode(false);
                    Some(camera.get_location())
                } else {
                    None
                }
            })
            .collect();

        // Activate cameras by the collected locations
        for loc in locations_to_activate {
            self.activate_cameras_by_camera_location(loc)?;
        }

        Ok(())
    }

    /// Recibe una location y activas todas las camaras que esten a menos de NIVEL_DE_PROXIMIDAD_MAXIMO de esta.
    ///
    /// Al activaralas se las pasa de modo ahorro de energia a modo activo
    ///
    /// Adicionalmente, cada camara activada despues avisa a las camaras colindantes para que,
    /// si estan a la distancia requerida, tambien se activen.
    fn activate_cameras_by_camera_location(
        &mut self,
        location: Location,
    ) -> Result<(), CameraError> {
        // Collect the locations that need to be activated first
        let mut locations_to_activate = Vec::new();

        for camera in self.cameras.values_mut() {
            let distancia = camera.get_location().distance(location.clone());
            if distancia <= NIVEL_DE_PROXIMIDAD_MAXIMO && camera.get_sleep_mode() {
                camera.set_sleep_mode(false);
                locations_to_activate.push(camera.get_location());
            }
        }

        // Activate cameras by the collected locations
        for loc in locations_to_activate {
            self.activate_cameras_by_camera_location(loc)?;
        }

        Ok(())
    }

    /// Recibe una location y desactiva todas las camaras que esten a menos de AREA_DE_ALCANCE de esta.
    ///
    /// Al desactivarlas se las pasa de modo activo a modo ahorro de energia
    ///
    /// Adicionalmente, cada camara desactivada despues avisa a las camaras colindantes para que,
    /// si estan a la distancia requerida, tambien se desactiven.
    pub fn deactivate_cameras(&mut self, location: Location) -> Result<(), CameraError> {
        // Collect the locations that need to be activated first
        let locations_to_activate: Vec<Location> = self
            .cameras
            .values_mut()
            .filter_map(|camera| {
                let distancia = camera.get_location().distance(location.clone());
                if distancia <= AREA_DE_ALCANCE {
                    camera.set_sleep_mode(true);
                    Some(camera.get_location())
                } else {
                    None
                }
            })
            .collect();

        // Activate cameras by the collected locations
        for loc in locations_to_activate {
            self.deactivate_cameras_by_camera_location(loc)?;
        }

        Ok(())
    }

    /// Recibe una location y desactiva todas las camaras que esten a menos de NIVEL_DE_PROXIMIDAD_MAXIMO de esta.
    ///
    /// Al desactivarlas se las pasa de modo activo a modo ahorro de energia
    ///
    /// Adicionalmente, cada camara desactivada despues avisa a las camaras colindantes para que,
    /// si estan a la distancia requerida, tambien se desactiven.
    fn deactivate_cameras_by_camera_location(
        &mut self,
        location: Location,
    ) -> Result<(), CameraError> {
        // Collect the locations that need to be activated first
        let mut locations_to_activate = Vec::new();

        for camera in self.cameras.values_mut() {
            let distancia = camera.get_location().distance(location.clone());
            if distancia <= NIVEL_DE_PROXIMIDAD_MAXIMO && !camera.get_sleep_mode() {
                camera.set_sleep_mode(true);
                locations_to_activate.push(camera.get_location());
            }
        }

        // Activate cameras by the collected locations
        for loc in locations_to_activate {
            self.deactivate_cameras_by_camera_location(loc)?;
        }

        Ok(())
    }

    /// Envia un mensaje mediante el channel para que el cliente lo envie al broker
    pub fn send_message(
        &mut self,
        message: Box<dyn MessagesConfig + Send>,
    ) -> Result<(), CameraError> {
        let lock = self.send_to_client_channel.lock().unwrap();
        match lock.send(message) {
            Ok(_) => {}
            Err(e) => {
                println!("Error sending message: {:?}", e);
                return Err(CameraError::SendError);
            }
        };
        Ok(())
    }
}

impl CameraSystem<Client> {
    pub fn with_real_client(address: String) -> Result<CameraSystem<Client>, ProtocolError> {
        CameraSystem::new(address, |rx, addr, config, tx| {
            Client::new(rx, addr, config, tx)
        })
    }
}

#[cfg(test)]

mod tests {
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;

    use crate::monitoring::incident::Incident;
    use crate::mqtt::broker::Broker;
    use crate::mqtt::client_message::ClientMessage;
    use crate::mqtt::publish::publish_config::PublishConfig;
    use crate::utils::incident_payload::IncidentPayload;

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

        thread::spawn(
            move || match CameraSystem::<Client>::with_real_client(addr.to_string()) {
                Ok(mut camera_system) => {
                    assert_eq!(camera_system.get_cameras().len(), 0);
                    assert_eq!(camera_system.get_camera_by_id(1), None);
                    assert_eq!(camera_system.get_camera(), None);
                }
                Err(e) => {
                    panic!("Error creating camera system: {:?}", e);
                }
            },
        );
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
            let mut camera_system =
                CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
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
            let mut camera_system =
                CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
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
            let mut camera_system =
                CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
            assert!(camera_system.run_client(None).is_ok());
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
            let mut camera_system =
                CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
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
            let mut camera_system =
                CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
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
                assert!(!camera.get_sleep_mode());
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
            let mut camera_system =
                CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
            let location = Location::new(5.0, 20.0);
            let id = camera_system.add_camera(location.clone()).unwrap();
            let location2 = Location::new(5.0, 2.0);
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
            assert!(camera_system.get_camera_by_id(id).unwrap().get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id2)
                .unwrap()
                .get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id3)
                .unwrap()
                .get_sleep_mode());
            assert!(camera_system
                .get_camera_by_id(id4)
                .unwrap()
                .get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id5)
                .unwrap()
                .get_sleep_mode());
        });
        match handle.join() {
            Ok(_) => {}
            Err(e) => {
                panic!("Error joining thread: {:?}", e);
            }
        }
    }

    #[test]

    fn test08_camara_lejana_se_activa_por_reaccion_en_cadena() {
        let args = vec!["127.0.0.1".to_string(), "5020".to_string()];
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
        let addr = "127.0.0.1:5020";
        let handle = thread::spawn(move || {
            let mut camera_system =
                CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
            let location = Location::new(10.0, 0.0);
            let id = camera_system.add_camera(location.clone()).unwrap();
            let location2 = Location::new(11.0, 0.0);
            let id2 = camera_system.add_camera(location2.clone()).unwrap();

            assert_eq!(camera_system.get_cameras().len(), 2);

            let incident_location = Location::new(0.0, 0.0);
            camera_system.activate_cameras(incident_location).unwrap();
            assert!(!camera_system.get_camera_by_id(id).unwrap().get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id2)
                .unwrap()
                .get_sleep_mode());
        });
        handle.join().unwrap();
    }

    #[test]
    fn test09_desactivar_camara() {
        let args = vec!["127.0.0.1".to_string(), "5037".to_string()];
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
        let addr = "127.0.0.1:5037";
        let handle = thread::spawn(move || {
            let mut camera_system =
                CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
            let location = Location::new(1.0, 2.0);
            let id = camera_system.add_camera(location.clone()).unwrap();
            assert_eq!(camera_system.get_cameras().len(), 1);
            assert_eq!(
                camera_system.get_camera_by_id(id).unwrap().get_location(),
                location
            );
            let incident_location = Location::new(1.0, 2.0);
            camera_system
                .activate_cameras(incident_location.clone())
                .unwrap();
            for camera in camera_system.get_cameras().values() {
                assert!(!camera.get_sleep_mode());
            }
            camera_system.deactivate_cameras(incident_location).unwrap();
            for camera in camera_system.get_cameras().values() {
                assert!(camera.get_sleep_mode());
            }
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_10_deactivate_multiple_cameras() {
        let args = vec!["127.0.0.1".to_string(), "5040".to_string()];
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
        let addr = "127.0.0.1:5040";
        let handle = thread::spawn(move || {
            let mut camera_system =
                CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
            let location = Location::new(5.0, 20.0);
            let id = camera_system.add_camera(location.clone()).unwrap();
            let location2 = Location::new(5.0, 2.0);
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
            camera_system
                .activate_cameras(incident_location.clone())
                .unwrap();
            assert!(camera_system.get_camera_by_id(id).unwrap().get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id2)
                .unwrap()
                .get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id3)
                .unwrap()
                .get_sleep_mode());
            assert!(camera_system
                .get_camera_by_id(id4)
                .unwrap()
                .get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id5)
                .unwrap()
                .get_sleep_mode());

            camera_system.deactivate_cameras(incident_location).unwrap();
            assert!(camera_system.get_camera_by_id(id).unwrap().get_sleep_mode());
            assert!(camera_system
                .get_camera_by_id(id2)
                .unwrap()
                .get_sleep_mode());
            assert!(camera_system
                .get_camera_by_id(id3)
                .unwrap()
                .get_sleep_mode());
            assert!(camera_system
                .get_camera_by_id(id4)
                .unwrap()
                .get_sleep_mode());
            assert!(camera_system
                .get_camera_by_id(id5)
                .unwrap()
                .get_sleep_mode());
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_11_desactivar_camara_por_proximidad() {
        let args = vec!["127.0.0.1".to_string(), "5017".to_string()];
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
        let addr = "127.0.0.1:5017";
        let handle = thread::spawn(move || {
            let mut camera_system =
                CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
            let location = Location::new(10.0, 0.0);
            let id = camera_system.add_camera(location.clone()).unwrap();
            let location2 = Location::new(11.0, 0.0);
            let id2 = camera_system.add_camera(location2.clone()).unwrap();

            assert_eq!(camera_system.get_cameras().len(), 2);

            let incident_location = Location::new(0.0, 0.0);
            camera_system.activate_cameras(incident_location).unwrap();
            assert!(!camera_system.get_camera_by_id(id).unwrap().get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id2)
                .unwrap()
                .get_sleep_mode());

            let incident_location = Location::new(0.0, 0.0);
            camera_system.deactivate_cameras(incident_location).unwrap();
            assert!(camera_system.get_camera_by_id(id).unwrap().get_sleep_mode());
            assert!(camera_system
                .get_camera_by_id(id2)
                .unwrap()
                .get_sleep_mode());
        });
        handle.join().unwrap();
    }
    #[test]
    fn test_run_client() {
        #[derive(Debug, Clone)]
        pub struct MockClient {
            messages: Vec<client_message::ClientMessage>,
        }

        impl ClientTrait for MockClient {
            fn client_run(&mut self) -> Result<(), ProtocolError> {
                Ok(())
            }

            fn clone_box(&self) -> Box<dyn ClientTrait> {
                Box::new(self.clone())
            }
        }

        impl MockClient {
            pub fn new(messages: Vec<client_message::ClientMessage>) -> MockClient {
                MockClient { messages }
            }

            pub fn send_messages(&self, sender: &Sender<client_message::ClientMessage>) {
                for message in &self.messages {
                    sender.send(message.clone()).unwrap();
                }
            }
        }

        let args = vec!["127.0.0.1".to_string(), "5006".to_string()];
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
        let address = "127.0.0.1:5006".to_string();

        let publish_config = PublishConfig::read_config(
            "./src/surveilling/publish_config_test.json",
            PayloadTypes::IncidentLocation(IncidentPayload::new(Incident::new(Location::new(
                1.0, 2.0,
            )))),
        )
        .unwrap();
        let publish = publish_config.parse_message(3);

        let messages = vec![publish];
        let mock_client = MockClient::new(messages.clone());

        let (tx2, rx2) = mpsc::channel();

        let mut camera_system =
            CameraSystem::<MockClient>::new(address.clone(), |_rx, _addr, _configg, _tx| {
                Ok(MockClient { messages })
            })
            .unwrap();

        //add cameras
        let location = Location::new(1.0, 1.0);
        camera_system.add_camera(location.clone());
        let location2 = Location::new(1.0, 2.0);
        camera_system.add_camera(location2.clone());
        let location3 = Location::new(1.0, 3.0);
        camera_system.add_camera(location3.clone());
        let location4 = Location::new(2.0, 5.0);
        camera_system.add_camera(location4.clone());

        let handle = thread::spawn(move || {
            mock_client.send_messages(&tx2);
            println!("meu deus");
            match camera_system.run_client(Some(rx2)) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error running client: {:?}", e);
                }
            }

            // Verify that cameras were activated as expected
            for camera in camera_system.get_cameras().values() {
                println!(
                    "camer: location {:?}, sleep: {:?}",
                    camera.get_location(),
                    camera.get_sleep_mode()
                );

                assert!(!camera.get_sleep_mode());
            }
        });

        handle.join().unwrap();
    }
}
