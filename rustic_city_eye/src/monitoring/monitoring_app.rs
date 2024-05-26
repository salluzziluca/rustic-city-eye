//! Se conecta mediante TCP a la dirección asignada por argv.
//! Lee lineas desde stdin y las manda mediante el socket.

use std::sync::mpsc::{self, Sender};

use crate::monitoring::incident::Incident;
use crate::mqtt::{
    client::Client, connect_properties, protocol_error::ProtocolError, will_properties,
};
use crate::surveilling::camera::Camera;
use crate::surveilling::{camera_system::*, location::Location};

#[derive(Debug)]
#[allow(dead_code)]
pub struct MonitoringApp {
    send_to_client_channel: Sender<String>,
    monitoring_app_client: Client,
    camera_system: CameraSystem,
    incidents: Vec<Incident>,
}

#[allow(dead_code)]
impl MonitoringApp {
    ///recibe una addres a la que conectarse
    /// Crea el cliente de la app de monitoreo y lo conecta al broker
    /// Crea un sistema de cámaras y agrega una cámara al sistema
    pub fn new(args: Vec<String>) -> Result<MonitoringApp, ProtocolError> {
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

        let address = args[0].to_string() + ":" + &args[1].to_string();
        let camera_system_args = vec![
            args[0].clone(),
            args[1].clone(),
            "camera_system".to_string(),
            "CamareandoCamaritas123".to_string(),
        ];

        let camera_system = CameraSystem::new(camera_system_args)?;

        let (tx, rx) = mpsc::channel();

        let monitoring_app = MonitoringApp {
            send_to_client_channel: tx,
            incidents: Vec::new(),
            camera_system,
            monitoring_app_client: match Client::new(
                rx,
                address,
                will_properties,
                connect_properties,
                true,
                true,
                1,
                true,
                args[2].clone(),
                args[3].clone(),
                35,
                "kvtr33".to_string(),
                "camera_system".to_string(),
                "soy el camera_system y me desconecté".to_string(),
            ) {
                Ok(client) => client,
                Err(err) => return Err(err),
            },
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
        println!("mis incidentes: {:?}", self.incidents);
        let publish_incident = "publish: dup:1 qos:1 retain:1 accidente juancitoccrack".to_string() + &incident.to_string();
        let _ = self.send_to_client_channel.send(publish_incident);
    }

    pub fn get_cameras(&self) -> Vec<Camera> {
        self.camera_system.get_cameras().clone()
    }
}
