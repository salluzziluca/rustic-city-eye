//! Se conecta mediante TCP a la dirección asignada por los args que le ingresan
//! en su constructor.

use std::sync::mpsc::{self, Sender};

use crate::drones::drone_system::DroneSystem;
use crate::monitoring::incident::Incident;
use crate::mqtt::client_message;
use crate::mqtt::messages_config::MessagesConfig;
use crate::mqtt::publish::publish_config::PublishConfig;
use crate::mqtt::publish::publish_properties::{PublishProperties, TopicProperties};
use crate::mqtt::{client::Client, protocol_error::ProtocolError};
use crate::surveilling::camera::Camera;
use crate::surveilling::camera_system::*;
use crate::utils::incident_payload::IncidentPayload;
use crate::utils::location::Location;
use crate::utils::payload_types::PayloadTypes;

#[derive(Debug)]
#[allow(dead_code)]
pub struct MonitoringApp {
    send_to_client_channel: Sender<Box<dyn MessagesConfig + Send>>,
    monitoring_app_client: Client,
    camera_system: CameraSystem,
    incidents: Vec<Incident>,
    drone_system: DroneSystem,
}

#[allow(dead_code)]
impl MonitoringApp {
    ///recibe una addres a la que conectarse
    /// Crea el cliente de la app de monitoreo y lo conecta al broker
    /// Crea un sistema de cámaras y agrega una cámara al sistema
    pub fn new(args: Vec<String>) -> Result<MonitoringApp, ProtocolError> {
        let connect_config =
            client_message::Connect::read_connect_config("./src/monitoring/connect_config.json")?;

        let address = args[2].to_string() + ":" + &args[3].to_string();
        let camera_system_args = vec![
            args[2].clone(),
            args[3].clone(),
            "camera_system".to_string(),
            "CamareandoCamaritasForever".to_string(),
        ];

        let camera_system = CameraSystem::new(camera_system_args)?;
        let drone_system = DroneSystem::new(
            "src/drone_system/drone_config.json".to_string(),
            address.clone(),
        );
        let (tx, rx) = mpsc::channel();

        let monitoring_app = MonitoringApp {
            send_to_client_channel: tx,
            incidents: Vec::new(),
            camera_system,
            monitoring_app_client: match Client::new(rx, address, connect_config) {
                Ok(client) => client,
                Err(err) => return Err(err),
            },
            drone_system,
        };

        Ok(monitoring_app)
    }

    pub fn run_client(&mut self) -> Result<(), ProtocolError> {
        self.monitoring_app_client.client_run()?;
        self.camera_system.run_client()?;
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
    ) -> Result<(), ProtocolError> {
        match self.drone_system.add_drone(location, drone_center_id) {
            Ok(_) => Ok(()),
            Err(e) => Err(ProtocolError::DroneError(e.to_string())),
        }
    }

    pub fn add_drone_center(&mut self, location: Location) -> u32 {
        self.drone_system.add_drone_center(location).map_or(0, |id| id)
    }
    pub fn get_cameras(&self) -> Vec<Camera> {
        self.camera_system.get_cameras().clone()
    }

    pub fn get_incidents(&self) -> Vec<Incident> {
        self.incidents.clone()
    }
}
