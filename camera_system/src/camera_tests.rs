#[cfg(test)]

mod tests {
    use std::collections::HashMap;
    
    
    use std::sync::mpsc::{self, Sender};
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;

    use broker::broker::Broker;
    use client::client::{Client, ClientTrait};
    use protocol::client_message::{self, ClientMessage};
    use protocol::messages_config::MessagesConfig;
    use protocol::publish::payload_types::PayloadTypes;
    use protocol::publish::publish_config::PublishConfig;
    use utils::incident::Incident;
    use utils::incident_payload::IncidentPayload;
    use utils::location::Location;
    use utils::protocol_error::ProtocolError;

    use crate::camera_system::CameraSystem;


    

    impl CameraSystem<Client> {
        fn get_client_publish_end_channel(
            &self,
        ) -> Arc<
            std::sync::Mutex<std::sync::mpsc::Receiver<Box<(dyn MessagesConfig + Send + 'static)>>>,
        > {
            self.camera_system_client.get_publish_end_channel()
        }
    }
    
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
                    assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 0);
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
            let (id, _) = camera_system.add_camera(location).unwrap();
            assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 1);
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
            let (id, _) = camera_system.add_camera(location).unwrap();
            assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 1);
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
            let camera_system: CameraSystem<Client> =
                CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
            let arc_system = Arc::new(Mutex::new(camera_system));
            assert!(CameraSystem::<Client>::run_client(None, arc_system).is_ok());
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
            let (id, _) = camera_system.add_camera(location).unwrap();
            assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 1);
            assert_eq!(
                camera_system.get_camera_by_id(id).unwrap().get_location(),
                location
            );
            assert_eq!(camera_system.get_camera().unwrap().get_location(), location);
            let incident_location = Location::new(1.0, 2.0);
            camera_system.activate_cameras(incident_location).unwrap();
            for camera in camera_system.get_cameras().lock().unwrap().values() {
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
            let id = camera_system.add_camera(location).unwrap();
            let location2 = Location::new(0.0001, 0.0002);
            let id2 = camera_system.add_camera(location2).unwrap();
            let location3 = Location::new(0.0003, 0.0001);
            let id3 = camera_system.add_camera(location3).unwrap();
            let location4 = Location::new(10.0, 20.0);
            let id4 = camera_system.add_camera(location4).unwrap();
            let location5 = Location::new(0.0001, 0.0002);
            let id5 = camera_system.add_camera(location5).unwrap();
            assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 5);
            assert_eq!(
                camera_system.get_camera_by_id(id.0).unwrap().get_location(),
                location
            );
            let incident_location = Location::new(0.0, 0.0);
            camera_system.activate_cameras(incident_location).unwrap();
            assert!(camera_system.get_camera_by_id(id.0).unwrap().get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id2.0)
                .unwrap()
                .get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id3.0)
                .unwrap()
                .get_sleep_mode());
            assert!(camera_system
                .get_camera_by_id(id4.0)
                .unwrap()
                .get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id5.0)
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
            let location = Location::new(10.0 / 10000.0, 0.0);
            let id = camera_system.add_camera(location).unwrap();
            let location2 = Location::new(11.0 / 10000.0, 0.0);
            let id2 = camera_system.add_camera(location2).unwrap();

            assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 2);

            let incident_location = Location::new(0.0, 0.0);
            camera_system.activate_cameras(incident_location).unwrap();
            assert!(!camera_system.get_camera_by_id(id.0).unwrap().get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id2.0)
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
            let id = camera_system.add_camera(location).unwrap();
            assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 1);
            assert_eq!(
                camera_system.get_camera_by_id(id.0).unwrap().get_location(),
                location
            );
            let incident_location = Location::new(1.0, 2.0);
            camera_system.activate_cameras(incident_location).unwrap();
            for camera in camera_system.get_cameras().lock().unwrap().values() {
                assert!(!camera.get_sleep_mode());
            }
            camera_system.deactivate_cameras(incident_location).unwrap();
            for camera in camera_system.get_cameras().lock().unwrap().values() {
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
            let location = Location::new(5.0 / 1000.0, 20.0 / 1000.0);
            let id = camera_system.add_camera(location).unwrap();
            let location2 = Location::new(5.0 / 10000.0, 2.0 / 10000.0);
            let id2 = camera_system.add_camera(location2).unwrap();
            let location3 = Location::new(10.0 / 10000.0, 2.0 / 10000.0);
            let id3 = camera_system.add_camera(location3).unwrap();
            let location4 = Location::new(10.0 / 1000.0, 20.0 / 1000.0);
            let id4 = camera_system.add_camera(location4).unwrap();
            let location5 = Location::new(1.0 / 10000.0, 2.0 / 10000.0);
            let id5 = camera_system.add_camera(location5).unwrap();
            assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 5);
            assert_eq!(
                camera_system.get_camera_by_id(id.0).unwrap().get_location(),
                location
            );
            let incident_location = Location::new(0.0, 0.0);
            camera_system.activate_cameras(incident_location).unwrap();
            assert!(camera_system.get_camera_by_id(id.0).unwrap().get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id2.0)
                .unwrap()
                .get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id3.0)
                .unwrap()
                .get_sleep_mode());
            assert!(camera_system
                .get_camera_by_id(id4.0)
                .unwrap()
                .get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id5.0)
                .unwrap()
                .get_sleep_mode());

            camera_system.deactivate_cameras(incident_location).unwrap();
            assert!(camera_system.get_camera_by_id(id.0).unwrap().get_sleep_mode());
            assert!(camera_system
                .get_camera_by_id(id2.0)
                .unwrap()
                .get_sleep_mode());
            assert!(camera_system
                .get_camera_by_id(id3.0)
                .unwrap()
                .get_sleep_mode());
            assert!(camera_system
                .get_camera_by_id(id4.0)
                .unwrap()
                .get_sleep_mode());
            assert!(camera_system
                .get_camera_by_id(id5.0)
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
            let location = Location::new(10.0 / 10000.0, 0.0);
            let id = camera_system.add_camera(location).unwrap();
            let location2 = Location::new(11.0 / 10000.0, 0.0);
            let id2 = camera_system.add_camera(location2).unwrap();

            assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 2);

            let incident_location = Location::new(0.0, 0.0);
            camera_system.activate_cameras(incident_location).unwrap();
            assert!(!camera_system.get_camera_by_id(id.0).unwrap().get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id2.0)
                .unwrap()
                .get_sleep_mode());

            let incident_location = Location::new(0.0, 0.0);
            camera_system.deactivate_cameras(incident_location).unwrap();
            assert!(camera_system.get_camera_by_id(id.0).unwrap().get_sleep_mode());
            assert!(camera_system
                .get_camera_by_id(id2.0)
                .unwrap()
                .get_sleep_mode());
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_12_multiples_incidentes() {
        let args = vec!["127.0.0.1".to_string(), "5066".to_string()];
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
        let addr = "127.0.0.1:5066";
        let handle = thread::spawn(move || {
            let mut cameras = HashMap::new();
            let mut camera_system =
                CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
            let channel = camera_system.get_client_publish_end_channel();
            let location = Location::new(10.0 / 10000.0, 0.0);
            let id = camera_system.add_camera(location).unwrap();
            let location2 = Location::new(11.0 / 10000.0, 0.0);
            let id2 = camera_system.add_camera(location2).unwrap();

            assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 2);

            let incident_location = Location::new(0.0, 0.0);
            camera_system.activate_cameras(incident_location).unwrap();
            assert!(!camera_system.get_camera_by_id(id.0).unwrap().get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id2.0)
                .unwrap()
                .get_sleep_mode());

            let incident_location = Location::new(0.0, 0.0);
            camera_system.activate_cameras(incident_location).unwrap();
            assert!(!camera_system.get_camera_by_id(id.0).unwrap().get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id2.0)
                .unwrap()
                .get_sleep_mode());

            let incident_location = Location::new(2.00001, 2.00001);
            let id3 = camera_system.add_camera(incident_location).unwrap();
            let id4 = camera_system.add_camera(incident_location).unwrap();
            camera_system.activate_cameras(incident_location).unwrap();
            assert!(!camera_system.get_camera_by_id(id.0).unwrap().get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id2.0)
                .unwrap()
                .get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id3.0)
                .unwrap()
                .get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id4.0)
                .unwrap()
                .get_sleep_mode());
            let message_config = channel.lock().unwrap().recv().unwrap();
            let message = message_config.parse_message(0);
            println!("mensaje 1{:?}", message);
            let message_config = channel.lock().unwrap().recv().unwrap();
            let message = message_config.parse_message(1);
            println!("mensaje 2{:?}", message);
            let message_config = channel.lock().unwrap().recv().unwrap();
            let message = message_config.parse_message(2);
            println!("mensaje 3{:?}", message);
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
                    if let PayloadTypes::CamerasUpdatePayload(updated_cameras) = payload {
                        for camera in updated_cameras {
                            cameras.insert(camera.get_id(), camera);
                        }
                    }
                }
                _ => {
                    panic!("Unexpected message type");
                }
            }
            let message_config = channel.lock().unwrap().recv().unwrap();
            let message = message_config.parse_message(3);
            println!("mensaje 4{:?}", message);
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
                    if let PayloadTypes::CamerasUpdatePayload(updated_cameras) = payload {
                        for camera in updated_cameras {
                            cameras.insert(camera.get_id(), camera);
                        }
                    }
                }
                _ => {
                    panic!("Unexpected message type");
                }
            }
            assert_eq!(cameras.len(), 4);
            for camera in cameras.values() {
                assert!(!camera.get_sleep_mode());
            }
        });
        handle.join().unwrap();
    }
    
    #[test]
    fn test_update_cameras() {
        let args = vec!["127.0.0.1".to_string(), "5033".to_string()];
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
        let addr = "127.0.0.1:5033";
        let handle = thread::spawn(move || {
            let mut camera_system =
                CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
            let reciever = camera_system.camera_system_client.get_publish_end_channel();
            let reciever = reciever.lock().unwrap();
            let message = reciever.recv().unwrap();
            //conver message to a ClientMessage
            let packet_id = camera_system.camera_system_client.assign_packet_id();
            let message = message.parse_message(packet_id);
            // Recibe el sub que hace el camera_system cuando se crea por primera vez
            match message {
                ClientMessage::Subscribe {
                    packet_id: _,
                    payload,
                    properties: _,
                } => {
                    assert_eq!(payload.topic, "incident");
                }
                _ => {
                    panic!("Unexpected message type");
                }
            }
            let message = reciever.recv().unwrap();
            //conver message to a ClientMessage
            let packet_id = camera_system.camera_system_client.assign_packet_id();
            let message = message.parse_message(packet_id);
            // Recibe el sub que hace el camera_system cuando se crea por primera vez
            match message {
                ClientMessage::Subscribe {
                    packet_id: _,
                    payload,
                    properties: _,
                } => {
                    assert_eq!(payload.topic, "incident_resolved");
                }
                _ => {
                    panic!("Unexpected message type");
                }
            }
            let location = Location::new(1.0, 2.0);
            let id = camera_system.add_camera(location).unwrap();
            assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 1);
            assert_eq!(
                camera_system.get_camera_by_id(id.0).unwrap().get_location(),
                location
            );
            let incident_location = Location::new(1.0, 2.0);
            camera_system.activate_cameras(incident_location).unwrap();
            for camera in camera_system.get_cameras().lock().unwrap().values() {
                assert!(!camera.get_sleep_mode());
            }

            let message = reciever.recv().unwrap();
            //conver message to a ClientMessage
            let packet_id = camera_system.camera_system_client.assign_packet_id();
            let message = message.parse_message(packet_id);

            match message {
                ClientMessage::Publish {
                    topic_name,
                    payload: PayloadTypes::CamerasUpdatePayload(cameras),
                    ..
                } => {
                    assert_eq!(topic_name, "camera_update");
                    assert_eq!(cameras.len(), 1);
                    assert_eq!(cameras[0].get_location(), location);
                    assert!(!cameras[0].get_sleep_mode());
                }
                _ => {
                    panic!("Unexpected message type: {:?}", message);
                }
            }

            camera_system.deactivate_cameras(incident_location).unwrap();
            for camera in camera_system.get_cameras().lock().unwrap().values() {
                assert!(camera.get_sleep_mode());
            }

            let message = reciever.recv().unwrap();
            //conver message to a ClientMessage
            let packet_id = camera_system.camera_system_client.assign_packet_id();
            let message = message.parse_message(packet_id);
            print!("AAAAAAAAAAAAa{:?}", message);

            match message {
                ClientMessage::Publish {
                    topic_name,
                    payload: PayloadTypes::CamerasUpdatePayload(cameras),
                    ..
                } => {
                    assert_eq!(topic_name, "camera_update");
                    assert_eq!(cameras.len(), 1);
                    assert_eq!(cameras[0].get_location(), location);
                    assert!(cameras[0].get_sleep_mode());
                }
                _ => {
                    panic!("Unexpected message type");
                }
            }
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
            fn assign_packet_id(&self) -> u16 {
                0
            }
            fn get_publish_end_channel(
                &self,
            ) -> Arc<
                std::sync::Mutex<
                    std::sync::mpsc::Receiver<Box<(dyn MessagesConfig + Send + 'static)>>,
                >,
            > {
                Arc::new(std::sync::Mutex::new(std::sync::mpsc::channel().1))
            }

            fn get_client_id(&self) -> String {
                "mock".to_string()
            }

            fn disconnect_client(&self) -> Result<(), ProtocolError> {
                Ok(())
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
            "camera_system/src/packets_config/publish_config_test.json",
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
        let _ = camera_system.add_camera(location);
        let location2 = Location::new(1.0, 2.0);
        let _ = camera_system.add_camera(location2);
        let location3 = Location::new(1.0, 3.0);
        let _ = camera_system.add_camera(location3);
        let location4 = Location::new(2.0, 5.0);
        let _ = camera_system.add_camera(location4);

        let handle = thread::spawn(move || {
            mock_client.send_messages(&tx2);
            println!("CameraSys: meu deus");
            let arc_system: Arc<Mutex<CameraSystem<Client>>> = Arc::new(Mutex::new(
                CameraSystem::<Client>::with_real_client(address.to_string()).unwrap(),
            ));
            let arc_sys_clone = Arc::clone(&arc_system);
            let arc_rx2 = Arc::new(Mutex::new(rx2));
            match CameraSystem::<Client>::run_client(Some(arc_rx2), arc_sys_clone) {
                Ok(_) => {}
                Err(e) => {
                    println!("CameraSys: Error running client: {:?}", e);
                }
            }
            let camera_system = arc_system.lock().unwrap();
            // Verify that cameras were activated as expected
            for camera in camera_system.get_cameras().lock().unwrap().values() {
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

    // #[test]
    // fn test_13_creo_dirs_y_al_hacer_disconnect_se_borran() {
    //     let args = vec!["127.0.0.1".to_string(), "6000".to_string()];
    //     let addr = "127.0.0.1:6000";
    //     let mut broker = match Broker::new(args) {
    //         Ok(broker) => broker,
    //         Err(e) => {
    //             panic!("Error creating broker: {:?}", e)
    //         }
    //     };
    //     thread::spawn(move || {
    //         _ = broker.server_run();
    //     });

    //     let handle = thread::spawn(move || {
    //         let mut camera_system =
    //             CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
    //         let location = Location::new(1.0, 2.0);
    //         let id = camera_system.add_camera(location).unwrap();
    //         let location2 = Location::new(1.0, 5.0);
    //         let id2 = camera_system.add_camera(location2).unwrap();
    //         assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 2);
    //         assert_eq!(
    //             camera_system.get_camera_by_id(id.0).unwrap().get_location(),
    //             location
    //         );
    //         assert_eq!(
    //             camera_system.get_camera_by_id(id2.0).unwrap().get_location(),
    //             location2
    //         );

    //         // verificar que el path /camera_system/src/surveilling/cameras/{} id existe


    //         let project_dir = env!("CARGO_MANIFEST_DIR");

            
    //         let file_path_1 = Path::new("cameras/").join(id.0.to_string());
    //         let file_path_1 = Path::new(project_dir).join(file_path_1);
    //         // eliminar el primer "camera_system encontrado en ese file_path_1"


    //         println!("file_path_1: {:?}", file_path_1);
    //         assert!(file_path_1.exists());

    //         let file_path_2 = Path::new("cameras/").join(id2.0.to_string());




    //         assert!(file_path_1.exists());
    //         assert!(file_path_2.exists());
                        

    //         camera_system.disconnect().unwrap();

    //         assert!(!Path::new(file_path_1.as_path()).exists());
    //         assert!(!Path::new(file_path_2.as_path()).exists());
    //     });
    //     handle.join().unwrap();
    // }

    // #[test]
    // fn test_creo_file_en_dir_e_imprime_ok() {
    //     let args = vec!["127.0.0.1".to_string(), "5055".to_string()];
    //     let addr = "127.0.0.1:5055";
    //     let mut broker = match Broker::new(args) {
    //         Ok(broker) => broker,
    //         Err(e) => panic!("Error creating broker: {:?}", e),
    //     };

    //     let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
    //     let server_ready_clone = server_ready.clone();
    //     thread::spawn(move || {
    //         {
    //             let (lock, cvar) = &*server_ready_clone;
    //             let mut ready = lock.lock().unwrap();
    //             *ready = true;
    //             cvar.notify_all();
    //         }
    //         let _ = broker.server_run();
    //     });

    //     // Wait for the server to start
    //     {
    //         let (lock, cvar) = &*server_ready;
    //         let mut ready = lock.lock().unwrap();
    //         while !*ready {
    //             ready = cvar.wait(ready).unwrap();
    //         }
    //     }
    //     let locurote = thread::spawn(move || {
    //         let camera_system = CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
    //         let camera_arc = Arc::new(Mutex::new(camera_system));
    //         let can_start_second_thread = Arc::new((Mutex::new(false), Condvar::new()));
    //         let second_thread_completed = Arc::new((Mutex::new(false), Condvar::new()));
    //         let camera_id_shared = Arc::new(Mutex::new(None)); // Shared camera ID

    //         let (
    //             can_start_second_thread_clone,
    //             second_thread_completed_clone,
    //             camera_id_shared_clone,
    //         ) = (
    //             Arc::clone(&can_start_second_thread),
    //             Arc::clone(&second_thread_completed),
    //             Arc::clone(&camera_id_shared),
    //         );

    //         // Thread 1: Camera system run
    //         let camera_arc_clone_for_thread1 = Arc::clone(&camera_arc);
    //         let camera_arc_clone_for_thread2 = Arc::clone(&camera_arc);
    //         let handler_camera_system = thread::spawn(move || {
    //             CameraSystem::<Client>::run_client(None, camera_arc_clone_for_thread1).unwrap();
    //             let mut camera_system = camera_arc_clone_for_thread2.lock().unwrap();
    //             let location = Location::new(1.0, 2.0);
    //             let id = camera_system.add_camera(location).unwrap();
    //             *camera_id_shared.lock().unwrap() = Some(id.0);
    //             // Signal to start the second thread
    //             {
    //                 let (lock, cvar) = &*can_start_second_thread_clone;
    //                 let mut start = lock.lock().unwrap();
    //                 *start = true;
    //                 cvar.notify_one();
    //             }
    //             // Wait for the second thread to complete
    //             {
    //                 let (lock, cvar) = &*second_thread_completed_clone;
    //                 let mut completed = lock.lock().unwrap();
    //                 while !*completed {
    //                     completed = cvar.wait(completed).unwrap();
    //                 }
    //             }
    //             camera_system.disconnect().unwrap();
    //         });

    //         // Thread 2: File creation and deletion
    //         let handler_file_operations = thread::spawn(move || {
    //             // Wait for the signal from the first thread to start
    //             {
    //                 let (lock, cvar) = &*can_start_second_thread;
    //                 let mut start = lock.lock().unwrap();
    //                 while !*start {
    //                     start = cvar.wait(start).unwrap();
    //                 }
    //             } // Retrieve the shared camera ID
    //             let camera_id = camera_id_shared_clone.lock().unwrap();
    //             let path1 =
    //                 "src/surveilling/cameras/".to_string() + &camera_id.unwrap().to_string();
    //             let dir_path = Path::new(path1.as_str());
    //             let temp_file_path = dir_path.join("temp_file.txt");
    //             File::create(&temp_file_path).expect("Failed to create temporary file");

    //             // Wait 2 seconds
    //             thread::sleep(Duration::from_secs(2));

    //             std::fs::remove_file(temp_file_path).expect("Failed to remove temporary file");
    //             {
    //                 let (lock, cvar) = &*second_thread_completed;
    //                 let mut completed = lock.lock().unwrap();
    //                 *completed = true;
    //                 cvar.notify_one();
    //             }
    //         });

    //         // Wait for both threads to complete
    //         handler_camera_system.join().unwrap();
    //         handler_file_operations.join().unwrap();
    //     });
    //     locurote.join().unwrap();
    // }
}