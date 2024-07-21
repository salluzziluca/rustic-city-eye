//! Se conecta mediante TCP a la dirección asignada por los args que le ingresan
//! en su constructor.

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::drones::drone_system::DroneSystem;
use crate::monitoring::incident::Incident;

use crate::mqtt::client::ClientTrait;
use crate::mqtt::client_message::Connect;
use crate::mqtt::disconnect_config::DisconnectConfig;
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

use crate::surveilling::{camera::Camera, camera_system::CameraSystem};
use crate::utils::incident_payload::IncidentPayload;
use crate::utils::location::Location;
use crate::utils::payload_types::PayloadTypes;

#[derive(Debug)]
#[allow(dead_code)]
pub struct MonitoringApp {
    send_to_client_channel: Arc<Mutex<Sender<Box<dyn MessagesConfig + Send>>>>,
    monitoring_app_client: Client,
    camera_system: Arc<Mutex<CameraSystem<Client>>>,
    incidents: Arc<Mutex<Vec<(Incident, u8)>>>,
    drone_system: DroneSystem,
    receive_from_client: Arc<Mutex<Receiver<ClientMessage>>>,
    active_drones: Arc<Mutex<HashMap<u32, Location>>>,
    cameras: Arc<Mutex<HashMap<u32, Camera>>>,
    disconnect_notification_sender: Sender<()>
}

#[allow(dead_code)]
impl MonitoringApp {
    /// recibe una address a la que conectarse
    /// Crea el cliente de la app de monitoreo y lo conecta al broker.
    /// Una vez creado su Client, se suscribe a sus topics de interes.
    /// Crea un sistema de cámaras y agrega una cámara al sistema
    pub fn new(
        args: Vec<String>,
        disconnect_notification_sender: Sender<()>,
    ) -> Result<MonitoringApp, ProtocolError> {
        let mut connect_config =
            client_message::Connect::read_connect_config("src/monitoring/connect_config.json")?;
        connect_config.username = Some(args[0].to_string());
        connect_config.password = Some(args[1].as_bytes().to_vec());

        let address = args[2].to_string() + ":" + &args[3].to_string();

        let camera_system = match CameraSystem::<Client>::with_real_client(address.clone()) {
            Ok(camera_system) => camera_system,
            Err(err) => return Err(err),
        };
        let drone_system =
            DroneSystem::new("src/drones/drone_config.json".to_string(), address.clone());

        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        let monitoring_app_client =
            MonitoringApp::create_client(rx, address, connect_config, tx2, tx.clone())?;

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
            disconnect_notification_sender
        };

        Ok(monitoring_app)
    }

    /// Crea el Client a traves del cual la MonitoringApp va a comunicarse con la red.
    ///
    /// Este Client se va a construir a partir de la configuracion brindada en su archivo de configuracion.
    ///
    /// Una vez conectado, se envian packets que van a suscribir a la aplicacion a sus topics de interes(ver metodo subscribe_to_topics).
    ///
    /// Retorna error en caso de fallar la creacion.
    fn create_client(
        receive_from_monitoring_channel: Receiver<Box<dyn MessagesConfig + Send>>,
        address: String,
        connect_config: Connect,
        send_to_monitoring_channel: Sender<ClientMessage>,
        send_from_monitoring_channel: Sender<Box<dyn MessagesConfig + Send>>,
    ) -> Result<Client, ProtocolError> {
        match Client::new(
            receive_from_monitoring_channel,
            address,
            connect_config.clone(),
            send_to_monitoring_channel,
        ) {
            Ok(client) => {
                println!("Soy la MonitoringApp, y mi Client se conecto exitosamente!");
                MonitoringApp::subscribe_to_topics(connect_config, send_from_monitoring_channel)?;
                Ok(client)
            }
            Err(_) => Err(ProtocolError::ConectionError),
        }
    }

    /// Contiene las subscripciones a los topics de interes para la MonitoringApp:
    /// necesita la suscripcion al topic de locations de drones("drone_locations"),
    /// al de actualizaciones de camaras(camera_update), y al topic de
    /// drones atendiendo incidentes:
    ///
    /// La idea es que la aplicacion reciba actualizaciones de estado de parte del camera_system,
    /// y de los Drones que tenga creados, y que pueda plasmar estos cambios en la interfaz grafica.
    fn subscribe_to_topics(
        connect_config: Connect,
        send_from_monitoring_channel: Sender<Box<dyn MessagesConfig + Send>>,
    ) -> Result<(), ProtocolError> {
        let subscribe_properties: SubscribeProperties =
            SubscribeProperties::new(1, connect_config.properties.user_properties);
        let topic_name = "drone_locations".to_string();

        let subscribe_config = SubscribeConfig::new(
            topic_name.clone(),
            subscribe_properties.clone(),
            connect_config.client_id.clone(),
        );
        match send_from_monitoring_channel.send(Box::new(subscribe_config)) {
            Ok(_) => {
                println!(
                    "Monitoring App subscrita al topic {} correctamente",
                    topic_name
                );
            }
            Err(e) => {
                println!("Monitoring: Error sending message: {:?}", e);
                return Err(ProtocolError::SubscribeError);
            }
        };

        let topic_name = "camera_update".to_string();
        let subscribe_config = SubscribeConfig::new(
            topic_name.clone(),
            subscribe_properties.clone(),
            connect_config.client_id.clone(),
        );
        match send_from_monitoring_channel.send(Box::new(subscribe_config)) {
            Ok(_) => {
                println!(
                    "Monitoring App subscrita al topic {} correctamente",
                    topic_name
                );
            }

            Err(e) => {
                println!("Monitoring: Error sending message: {:?}", e);
                return Err(ProtocolError::SubscribeError);
            }
        };

        let topic_name = "incidente_resuelto".to_string();
        let subscribe_config = SubscribeConfig::new(
            topic_name.clone(),
            subscribe_properties,
            connect_config.client_id,
        );
        match send_from_monitoring_channel.send(Box::new(subscribe_config)) {
            Ok(_) => {
                println!(
                    "Monitoring App subscrita al topic {} correctamente",
                    topic_name
                );
            }
            Err(e) => {
                println!("Monitoring: Error sending message: {:?}", e);
                return Err(ProtocolError::SubscribeError);
            }
        };

        Ok(())
    }

    /// Se encarga de correr al Client de la MonitoringApp, y al Client del CameraSystem.
    pub fn run_client(&mut self) -> Result<(), ProtocolError> {
        let receive_from_client_ref = Arc::clone(&self.receive_from_client);
        let active_drones_clone = Arc::clone(&self.active_drones);
        let cameras_clone = Arc::clone(&self.cameras);
        let incidents_clone = Arc::clone(&self.incidents);

        thread::spawn(move || loop {
            let receiver_clone: Arc<Mutex<Receiver<ClientMessage>>> =
                Arc::clone(&receive_from_client_ref);
            let active_drones_clone: Arc<Mutex<HashMap<u32, Location>>> =
                Arc::clone(&active_drones_clone);
            let cameras_clone = Arc::clone(&cameras_clone);
            let incidents_clone = Arc::clone(&incidents_clone);

            if !update_entities(
                receiver_clone,
                active_drones_clone,
                cameras_clone,
                incidents_clone,
            ) {
                break;
            };
        });

        println!("Client de la monitoring app comienza a correr");
        self.monitoring_app_client.client_run()?;

        println!("Client del camera system comienza a correr");
        CameraSystem::<Client>::run_client(None, self.camera_system.clone())?;

        Ok(())
    }

    /// Dada una location para agregar una nueva camara, delega al camera_system la tarea de agregar la camara.
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

    /// Publica un nuevo incidente en una location determinada. La idea es que envie un packet
    /// del tipo Publish hacia el topic de incidente, y que tanto el camera_system como los Drones
    /// reciban la notificacion de este nuevo incidente.
    pub fn add_incident(&mut self, location: Location) {
        let incident = Incident::new(location);
        let mut incidents = match self.incidents.lock() {
            Ok(incidents) => incidents,
            Err(_) => return,
        };

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
            PublishConfig::new(1, 1, 1, "incidente".to_string(), payload, properties);
        let send_to_client_channel = match self.send_to_client_channel.lock() {
            Ok(send_to_client_channel) => send_to_client_channel,
            Err(_) => return,
        };

        match send_to_client_channel.send(Box::new(publish_config)) {
            Ok(_) => {
                println!("Incidente publicado correctamente");
            }
            Err(_) => {
                println!("Error al publicar el incidente");
            }
        }
    }

    /// Agrega un Drone: delega al DroneSystem la tarea de agregarlo efectivamente.
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

    /// Agrega un centro de Drones nuevo.
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

    /// Desconecta a los clientes de la MonitoringApp, del CameraSystem, y de los Drones(esto
    /// ultimo se hace a traves del DroneSystem).
    pub fn disconnect(&mut self) -> Result<(), ProtocolError> {
        let disconnect_config = DisconnectConfig::new(
            0x00_u8,
            1,
            "normal".to_string(),
            self.monitoring_app_client.get_client_id(),
        );
        let send_to_client_channel = match self.send_to_client_channel.lock() {
            Ok(send_to_client_channel) => send_to_client_channel,
            Err(_) => return Err(ProtocolError::LockError),
        };

        match send_to_client_channel.send(Box::new(disconnect_config)) {
            Ok(_) => {}
            Err(_) => {
                println!("Error al desconectar la Monitoring App");
                return Err(ProtocolError::DisconnectError);
            }
        }

        let system = match self.camera_system.lock() {
            Ok(system) => system,
            Err(_) => return Err(ProtocolError::LockError),
        };

        system.disconnect()?;
        self.drone_system.disconnect_system()?;

        println!("Cliente de la monitoring app desconectado correctamente");
        // exit(0);

        Ok(())
    }
}

/// Se encarga de recibir los mensajes publicados en los 3 topics a los que esta suscrito,
/// y actualiza el estado de las camaras y de los Drones que posea.
///
/// Si se recibe un mensaje del topic drone_locations, se actualiza la location del Drone que
/// indica el id del payload(esto lo captura la UI, y muestra al Drone en esa nueva posicion).
///
/// Si se recibe un mensaje del topic camera_update, se recibe un snapshot de las camaras que cambiaron su
/// estado, por lo que se notifica a la UI estos cambios y los muestra.
///
/// Si se recibe un mensaje del topic attendingincident, se lleva a cabo un procedimiento:
/// - La Monitoring App lleva un registro de todos los incidentes ingresados, junto a la cantidad de Drones que
///   estan resolviendolo(apenas se publica un incidente nuevo, este valor sera 0).
/// - A medida que vaya recibiendo mensajes sobre ese topic, se ira incrementando el valor de este contador.
/// - Si este contador llega a 2, se publica un mensaje con topic incidente_resuelto, el cual recibiran los Drones,
///   y aquellos que no fueron a resolver el incidente dejaran de ir a resolverlo, y volveran a patrullar en su area.
pub fn update_entities(
    recieve_from_client: Arc<Mutex<Receiver<ClientMessage>>>,
    active_drones: Arc<Mutex<HashMap<u32, Location>>>,
    cameras: Arc<Mutex<HashMap<u32, Camera>>>,
    incidents: Arc<Mutex<Vec<(Incident, u8)>>>,
    // disconnect_sender: Arc<Mutex<Sender<()>>>,
) -> bool {
    let receiver = match recieve_from_client.lock() {
        Ok(receiver) => receiver,
        Err(_) => return true,
    };

    if let Ok(ClientMessage::Publish {
        packet_id: _,
        topic_name,
        qos: _,
        retain_flag: _,
        payload,
        dup_flag: _,
        properties: _,
    }) = receiver.try_recv()
    {
        if topic_name == "drone_locations" {
            let mut active_drones: std::sync::MutexGuard<HashMap<u32, Location>> =
                match active_drones.try_lock() {
                    Ok(active_drones) => active_drones,
                    Err(_) => return true,
                };
            if let PayloadTypes::DroneLocation(id, drone_locationn) = payload {
                active_drones.insert(id, drone_locationn);
            }
        } else if topic_name == "camera_update" {
            println!("Monitoring: Camera update received");
            let mut cameras = match cameras.try_lock() {
                Ok(cameras) => cameras,
                Err(_) => return true,
            };
            if let PayloadTypes::CamerasUpdatePayload(updated_cameras) = payload {
                for camera in updated_cameras {
                    cameras.insert(camera.get_id(), camera);
                }
            }
        } else if topic_name == "incidente_resuelto" {
            if let PayloadTypes::IncidentLocation(incident_payload) = payload {
                let mut incidents = match incidents.lock() {
                    Ok(incidents) => incidents,
                    Err(_) => return true,
                };
                let mut to_remove = Vec::new();

                for (incident, _count) in incidents.iter_mut() {
                    if incident.get_location() == incident_payload.get_incident().get_location() {
                        to_remove.push(incident.clone());
                    }
                }

                incidents.retain(|(inc, _)| !to_remove.contains(inc));
            }
        }
    }

    if let Ok(ClientMessage::Disconnect {
        reason_code: _,
        session_expiry_interval: _,
        reason_string: _,
        client_id: _,
    }) = receiver.try_recv()
    {
        println!("aqui la aplicacion sabe que se debe desconectar");
        return false;
    }

    true
}
