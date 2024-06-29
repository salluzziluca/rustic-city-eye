//! Se conecta mediante TCP a la dirección asignada por los args que le ingresan
//! en su constructor.

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::drones::drone_system::DroneSystem;
use crate::monitoring::incident::Incident;

use crate::mqtt::client::ClientTrait;
use crate::mqtt::{
    client_message::{self, ClientMessage},
    messages_config::MessagesConfig,
    publish::{
        publish_config::PublishConfig,
        publish_properties::{PublishProperties, TopicProperties},
    },
    subscribe_config::SubscribeConfig,
    subscribe_properties::SubscribeProperties,
    {client::Client, protocol_error::ProtocolError},
};

use crate::surveilling::camera::Camera;
use crate::surveilling::camera_system::CameraSystem;
use crate::utils::incident_payload::IncidentPayload;
use crate::utils::location::Location;
use crate::utils::payload_types::PayloadTypes;

#[derive(Debug)]
#[allow(dead_code)]
pub struct MonitoringApp {
    send_to_client_channel: Arc<Mutex<Sender<Box<dyn MessagesConfig + Send>>>>,
    monitoring_app_client: Client,
    camera_system: Arc<Mutex<CameraSystem<Client>>>,
    // incidents: Arc<Mutex<Vec<Incident>>>,
    incidents: Arc<Mutex<Vec<(Incident, u8)>>>,
    drone_system: DroneSystem,
    receive_from_client: Arc<Mutex<Receiver<ClientMessage>>>,
    active_drones: Arc<Mutex<HashMap<u32, Location>>>,
    cameras: Arc<Mutex<HashMap<u32, Camera>>>,
}

#[allow(dead_code)]
impl MonitoringApp {
    ///recibe una addres a la que conectarse
    /// Crea el cliente de la app de monitoreo y lo conecta al broker
    /// Crea un sistema de cámaras y agrega una cámara al sistema
    pub fn new(args: Vec<String>) -> Result<MonitoringApp, ProtocolError> {
        let connect_config =
            client_message::Connect::read_connect_config("src/monitoring/connect_config.json")?;

        let address = args[2].to_string() + ":" + &args[3].to_string();
        let camera_system = match CameraSystem::<Client>::with_real_client(address.clone()) {
            Ok(camera_system) => camera_system,
            Err(err) => return Err(err),
        };
        let drone_system =
            DroneSystem::new("src/drones/drone_config.json".to_string(), address.clone());
        type MessagesConfigSender = Sender<Box<dyn MessagesConfig + Send>>;
        type MessagesConfigReceiver = Receiver<Box<dyn MessagesConfig + Send>>;
        let (tx, rx): (MessagesConfigSender, MessagesConfigReceiver) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        let monitoring_app_client = match Client::new(rx, address, connect_config, tx2) {
            Ok(client) => client,
            Err(err) => return Err(err),
        };

        let client_id = monitoring_app_client.get_client_id();
        let subscribe_properties: SubscribeProperties = SubscribeProperties::new(1, Vec::new());
        let subscribe_config = SubscribeConfig::new(
            "drone_locations".to_string(),
            1,
            subscribe_properties,
            client_id.clone(),
        );
        match tx.send(Box::new(subscribe_config)) {
            Ok(_) => {}
            Err(e) => {
                println!("Monitoring: Error sending message: {:?}", e);
                return Err(ProtocolError::SubscribeError);
            }
        };

        let subscribe_properties: SubscribeProperties = SubscribeProperties::new(1, Vec::new());
        let subscribe_config = SubscribeConfig::new(
            "camera_update".to_string(),
            1,
            subscribe_properties,
            client_id.clone(),
        );
        match tx.send(Box::new(subscribe_config)) {
            Ok(_) => {}
            Err(e) => {
                println!("Monitoring: Error sending message: {:?}", e);
                return Err(ProtocolError::SubscribeError);
            }
        };

        let subscribe_properties: SubscribeProperties = SubscribeProperties::new(1, Vec::new());
        let subscribe_config = SubscribeConfig::new(
            "attendingincident".to_string(),
            1,
            subscribe_properties,
            client_id.clone(),
        );
        match tx.send(Box::new(subscribe_config)) {
            Ok(_) => {}
            Err(e) => {
                println!("Monitoring: Error sending message: {:?}", e);
                return Err(ProtocolError::SubscribeError);
            }
        };

        let receive_from_client = Arc::new(Mutex::new(rx2));
        let active_drones = Arc::new(Mutex::new(HashMap::new()));
        let incidents = Arc::new(Mutex::new(Vec::new()));
        let cameras = Arc::new(Mutex::new(HashMap::new()));
        let tx_arc = Arc::new(Mutex::new(tx));
        let monitoring_app = MonitoringApp {
            send_to_client_channel: Arc::clone(&tx_arc),
            incidents: Arc::clone(&incidents),
            camera_system: Arc::new(Mutex::new(camera_system)),
            monitoring_app_client,
            drone_system,
            receive_from_client: Arc::clone(&receive_from_client),
            active_drones: Arc::clone(&active_drones),
            cameras: Arc::clone(&cameras),
        };
        thread::spawn(move || loop {
            let receiver_clone = Arc::clone(&receive_from_client);
            let active_drones_clone = Arc::clone(&active_drones);
            let cameras_clone = Arc::clone(&cameras);
            let incidents_clone = Arc::clone(&incidents);
            let tx_clone = Arc::clone(&tx_arc);

            update_entities(
                receiver_clone,
                active_drones_clone,
                cameras_clone,
                incidents_clone,
                tx_clone,
            );
        });
        Ok(monitoring_app)
    }

    pub fn run_client(&mut self) -> Result<(), ProtocolError> {
        self.monitoring_app_client.client_run()?;
        let _ = CameraSystem::<Client>::run_client(None, self.camera_system.clone());

        // let reciever_clone = Arc::clone(&self.recieve_from_client.clone());

        // thread::spawn(move || loop {
        //     let lock = reciever_clone.lock().unwrap();
        //     match lock.recv() {
        //         Ok(message) => {
        //             println!("Monitoring: Message received en monitoriung: {:?}", message);
        //         }
        //         Err(e) => {
        //             println!("Monitoring: Error receiving message: {:?}", e);
        //         }
        //     }
        // });
        Ok(())
    }

    pub fn add_camera(&mut self, location: Location) -> Result<u32, ProtocolError> {
        let mut lock = match self.camera_system.lock() {
            Ok(lock) => lock,
            Err(e) => {
                println!("Monitoring: Error locking camera system: {:?}", e);
                return Err(ProtocolError::LockError);
            }
        };
        match lock.add_camera(location) {
            Ok(id) => Ok(id),
            Err(e) => {
                println!("Monitoring: Error adding camera: {:?}", e);
                Err(ProtocolError::CameraError(e.to_string()))
            }
        }
    }

    pub fn add_incident(&mut self, location: Location) {
        let incident = Incident::new(location);
        let mut incidents = self.incidents.lock().unwrap();

        let (incident, drones_attending_incident) = (incident.clone(), 0);

        incidents.push((incident.clone(), drones_attending_incident));

        let topic_properties = TopicProperties {
            topic_alias: 10,
            response_topic: "".to_string(),
        };

        let properties = PublishProperties::new(
            1,
            10,
            topic_properties,
            [1, 2, 3].to_vec(),
            "a".to_string(),
            1,
            "a".to_string(),
        );
        let payload = PayloadTypes::IncidentLocation(IncidentPayload::new(incident));

        let publish_config =
            PublishConfig::new(1, 1, 0, "incidente".to_string(), payload, properties);
        let send_to_client_channel: std::sync::MutexGuard<Sender<Box<dyn MessagesConfig + Send>>> =
            self.send_to_client_channel.lock().unwrap();

        let _ = send_to_client_channel.send(Box::new(publish_config));
    }

    pub fn add_drone(
        &mut self,
        location: Location,
        drone_center_id: u32,
    ) -> Result<u32, ProtocolError> {
        match self.drone_system.add_drone(location, drone_center_id) {
            Ok(id) => Ok(id),
            Err(e) => Err(ProtocolError::DroneError(e.to_string())),
        }
    }

    pub fn add_drone_center(&mut self, location: Location) -> u32 {
        self.drone_system
            .add_drone_center(location)
            .map_or(0, |id| id)
    }

    pub fn get_incidents(&self) -> Vec<Incident> {
        match self.incidents.lock() {
            Ok(incidents) => {
                let mut incidents_without_drones = Vec::new();

                for (incident, _drone_amount) in incidents.iter() {
                    incidents_without_drones.push(incident.clone());
                }

                incidents_without_drones
            }
            Err(_) => Vec::new(),
        }
    }

    pub fn get_active_drones(&self) -> HashMap<u32, Location> {
        match self.active_drones.lock() {
            Ok(active_drones) => active_drones.clone(),
            Err(_) => HashMap::new(),
        }
    }

    pub fn get_cameras(&self) -> HashMap<u32, Camera> {
        match self.cameras.lock() {
            Ok(cameras) => cameras.clone(),
            Err(_) => HashMap::new(),
        }
    }
}

pub fn update_entities(
    recieve_from_client: Arc<Mutex<Receiver<ClientMessage>>>,
    active_drones: Arc<Mutex<HashMap<u32, Location>>>,
    cameras: Arc<Mutex<HashMap<u32, Camera>>>,
    incidents: Arc<Mutex<Vec<(Incident, u8)>>>,
    send_to_client_channel: Arc<Mutex<Sender<Box<dyn MessagesConfig + Send>>>>,
) {
    let receiver = match recieve_from_client.lock() {
        Ok(receiver) => receiver,
        Err(_) => return,
    };

    loop {
        if let Ok(message) = receiver.try_recv() {
            match message {
                ClientMessage::Publish {
                    packet_id: _,
                    topic_name,
                    qos: _,
                    retain_flag: _,
                    payload,
                    dup_flag: _,
                    properties: _,
                } => {
                    if topic_name == "drone_locations" {
                        let mut active_drones: std::sync::MutexGuard<HashMap<u32, Location>> =
                            match active_drones.try_lock() {
                                Ok(active_drones) => active_drones,
                                Err(_) => return,
                            };
                        if let PayloadTypes::DroneLocation(id, drone_locationn) = payload {
                            active_drones.insert(id, drone_locationn);
                        }
                    } else if topic_name == "camera_update" {
                        println!("Monitoring: Camera update received");
                        let mut cameras = match cameras.try_lock() {
                            Ok(cameras) => cameras,
                            Err(_) => return,
                        };
                        if let PayloadTypes::CamerasUpdatePayload(updated_cameras) = payload {
                            for camera in updated_cameras {
                                cameras.insert(camera.get_id(), camera);
                            }
                        }
                    } else if topic_name == "attendingincident" {
                        if let PayloadTypes::AttendingIncident(incident_payload) = payload {
                            let mut incidents = incidents.lock().unwrap();

                            let mut to_remove = Vec::new();

                            for (incident, count) in incidents.iter_mut() {
                                if incident.get_location()
                                    == incident_payload.get_incident().get_location()
                                {
                                    *count += 1;

                                    if *count == 2 {
                                        to_remove.push(incident.clone());
                                        //publish incident resolver message
                                        let incident_payload = IncidentPayload::new(Incident::new(
                                            incident.get_location(),
                                        ));
                                        let publish_config = match PublishConfig::read_config(
                                            "src/drones/publish_attending_incident_config.json",
                                            PayloadTypes::AttendingIncident(incident_payload),
                                        ) {
                                            Ok(config) => config,
                                            Err(e) => {
                                                println!("Error reading publish config: {:?}", e);
                                                return;
                                            }
                                        };
                                        let send_to_client_channel: std::sync::MutexGuard<
                                            Sender<Box<dyn MessagesConfig + Send>>,
                                        > = send_to_client_channel.lock().unwrap();

                                        match send_to_client_channel.send(Box::new(publish_config))
                                        {
                                            Ok(_) => {
                                                println!("AAAAAAA ENVIO LAS COSASSSSSS");
                                            }
                                            Err(e) => {
                                                println!(
                                                    "Error sending to client channel: {:?}",
                                                    e
                                                );
                                            }
                                        };
                                    }
                                }
                            }

                            // Remove incidents marked for removal
                            incidents.retain(|(inc, _)| !to_remove.contains(inc));

                            println!("Incidents: {:?}", incidents);
                        }
                    }
                }
                ClientMessage::Auth {
                    reason_code: _,
                    authentication_data: _n_data,
                    reason_string: _,
                    user_properties: _,
                    authentication_method: _,
                } => {
                    todo!()
                }
                _ => {}
            }
        }
    }
}
