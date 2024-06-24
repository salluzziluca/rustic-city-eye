//! Se conecta mediante TCP a la dirección asignada por los args que le ingresan
//! en su constructor.

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};

use crate::drones::drone_system::DroneSystem;
use crate::monitoring::incident::Incident;

use crate::mqtt::{
    client_message::{self, ClientMessage},
    messages_config::MessagesConfig,
    publish::{publish_properties::{PublishProperties, TopicProperties}, publish_config::PublishConfig},
    subscribe_config::SubscribeConfig,
    subscribe_properties::SubscribeProperties,
    {client::Client, protocol_error::ProtocolError}
};

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
        let drone_system = DroneSystem::new(
            "src/drone_system/drone_config.json".to_string(),
            address.clone(),
        );
        let (tx, rx): (
            Sender<Box<dyn MessagesConfig + Send>>,
            Receiver<Box<dyn MessagesConfig + Send>>,
        ) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

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
        };

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
        match self.camera_system.add_camera(location) {
            Ok(_) => {}
            Err(e) => {
                println!("Error adding camera: {:?}", e);
            }
        }
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

    pub fn update_drone_location(&self) -> HashMap<u32, Location> {
        let mut drone_locations = HashMap::new();
        loop {
            match self.recieve_from_client.try_recv() {
                Ok(message) => match message {
                    ClientMessage::Publish {
                        packet_id: _,
                        topic_name,
                        qos: _,
                        retain_flag: _,
                        payload,
                        dup_flag: _,
                        properties: _,
                    } => {
                        if topic_name == "drone_location" {
                            if let PayloadTypes::DroneLocation(id, drone_locationn) = payload {
                                drone_locations.insert(id, drone_locationn);
                                println!("Updated drone location");
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
                },
                Err(_) => {
                    return drone_locations;
                }
            }
        }
    }
}
