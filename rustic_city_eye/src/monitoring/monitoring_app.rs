//! Se conecta mediante TCP a la dirección asignada por los args que le ingresan
//! en su constructor.

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::drones::drone_location::DroneLocation;
use crate::drones::drone_system::DroneSystem;
use crate::monitoring::incident::Incident;
use crate::mqtt::client_message::{self, ClientMessage};
use crate::mqtt::messages_config::MessagesConfig;
use crate::mqtt::publish::publish_config::PublishConfig;
use crate::mqtt::publish::publish_properties::{PublishProperties, TopicProperties};
use crate::mqtt::subscribe_config::SubscribeConfig;
use crate::mqtt::subscribe_properties::SubscribeProperties;
use crate::mqtt::{client::Client, protocol_error::ProtocolError};
use crate::surveilling::camera::Camera;
use crate::surveilling::camera_system::CameraSystem;
use crate::utils::incident_payload::IncidentPayload;
use crate::utils::location::Location;
use crate::utils::payload_types::PayloadTypes;

#[derive(Debug)]
#[allow(dead_code)]
pub struct MonitoringApp {
    send_to_client_channel: Sender<Box<dyn MessagesConfig + Send>>,
    monitoring_app_client: Client,
    camera_system: CameraSystem<Client>,
    incidents: Vec<Incident>,
    drone_system: DroneSystem,
    recieve_from_client: Receiver<ClientMessage>,
    receive_from_drone_system: Arc<Mutex<Receiver<DroneLocation>>>,
    active_drones: Arc<Mutex<HashMap<u32, Location>>>,
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
        let (tx3, rx3) = mpsc::channel();

        let drone_system = DroneSystem::new(
            "src/drones/drone_config.json".to_string(),
            address.clone(),
            tx3,
        );
        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        let receive_from_drone_system = Arc::new(Mutex::new(rx3));
        let active_drones = Arc::new(Mutex::new(HashMap::new()));
        let monitoring_app = MonitoringApp {
            send_to_client_channel: tx,
            incidents: Vec::new(),
            camera_system,
            monitoring_app_client: match Client::new(rx, address, connect_config, tx2) {
                Ok(client) => client,
                Err(err) => return Err(err),
            },
            drone_system,
            recieve_from_client: rx2,
            receive_from_drone_system: receive_from_drone_system.clone(),
            active_drones: active_drones.clone(),
        };

        thread::spawn(move || {
            loop {
                let receiver = receive_from_drone_system.lock().unwrap();
                let mut drone_locations = active_drones.lock().unwrap();
                match receiver.try_recv() {
                    Ok(location) => {
                        drone_locations.insert(location.id, location.location);
                        println!("Updated drone location");
                    }
                    Err(e) => match e {
                        mpsc::TryRecvError::Empty => {}
                        mpsc::TryRecvError::Disconnected => {
                            println!("Disconnected from drone system");
                            break;
                        }
                    },
                }
            }
        });

        Ok(monitoring_app)
    }

    pub fn run_client(&mut self) -> Result<(), ProtocolError> {
        self.monitoring_app_client.client_run()?;
        self.camera_system.run_client(None)?;

        let subscribe_properties = SubscribeProperties::new(1, Vec::new());
        let subscribe_config =
            SubscribeConfig::new("drone_location".to_string(), 1, subscribe_properties);
        match self.send_to_client_channel.send(Box::new(subscribe_config)) {
            Ok(_) => {}
            Err(e) => {
                println!("Error sending message: {:?}", e);
                return Err(ProtocolError::SubscribeError);
            }
        };

        Ok(())
    }

    pub fn add_camera(&mut self, location: Location) {
        self.camera_system.add_camera(location);
    }

    pub fn add_incident(&mut self, location: Location) {
        let incident = Incident::new(location);
        self.incidents.push(incident.clone());

        let topic_properties = TopicProperties {
            topic_alias: 10,
            response_topic: "String".to_string(),
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

        let _ = self.send_to_client_channel.send(Box::new(publish_config));
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
    pub fn get_cameras(&self) -> HashMap<u32, Camera> {
        self.camera_system.get_cameras().clone()
    }

    pub fn get_incidents(&self) -> Vec<Incident> {
        self.incidents.clone()
    }

    pub fn get_drones(&self) -> HashMap<u32, Location> {
        self.active_drones.lock().unwrap().clone()
    }

    // pub fn update_drone_location(&self) -> HashMap<u32, Location> {
    //     let mut drone_locations = HashMap::new();
    //     loop {
    //         match self.receive_from_drone_system.try_recv() {
    //             Ok(location) => {
    //                 drone_locations.insert(location.id, location.location);
    //                 println!("Updated drone location");
    //             }
    //             Err(e) => {
    //                 match e {
    //                     mpsc::TryRecvError::Empty => {}
    //                     mpsc::TryRecvError::Disconnected => {
    //                         println!("Disconnected from drone system");
    //                     }
    //                 }
    //                 return drone_locations;
    //             }
    //         }
    //     }
    // }
}
