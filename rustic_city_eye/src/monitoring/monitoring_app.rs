//! Se conecta mediante TCP a la dirección asignada por los args que le ingresan
//! en su constructor.

use std::collections::{HashMap, VecDeque};
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
    publish::publish_config::PublishConfig,
    subscribe_config::SubscribeConfig,
    subscribe_properties::SubscribeProperties,
    {client::Client, protocol_error::ProtocolError},
};

use crate::surveilling::{camera::Camera, camera_system::CameraSystem};
use crate::utils::incident_payload::IncidentPayload;
use crate::utils::location::Location;
use crate::utils::payload_types::PayloadTypes;

/// Es capaz de recibir la carga de incidentes por parte del usuario(que lo hace desde la interfaz grafica)
/// y notificar a la red ante la aparicion de un incidente nuevo y los cambios de estado del mismo.
///
/// A traves de la interfaz grafica se visualiza el estado completo del sistema, incluyendo un mapa geografico de la region
/// con los incidentes activos y la posicion y estado de cada Drone y cada camara de vigilancia.
///
/// Se comunica con la red(es decir, recibe y envia mensajes) a traves de un Client de MQTT: a traves de este se sabran los cambios de
/// estado de cada agente e incidente, ademas de saber los cambios de estado en el sistema central de camaras.
#[derive(Debug)]
#[allow(dead_code)]
pub struct MonitoringApp {
    /// Es el Client de MQTT con el cual se comunicara en la red. Al crear una instancia de MonitoringApp, se creara
    /// tambien un Client para comenzar a intercambiar mensajes con la red.
    monitoring_app_client: Client,

    /// A traves de este canal se envian configuraciones de los packets a enviar desde la aplicacion a la red.
    send_to_client_channel: Arc<Mutex<Sender<Box<dyn MessagesConfig + Send>>>>,

    /// Es la instancia del sistema central de camaras del sistema: el mismo tiene la ubicacion y estado de cada camara, y
    /// es capaz de agregar, quitar y modificar las mismas.
    camera_system: Arc<Mutex<CameraSystem<Client>>>,

    /// Aqui se guardan los incidentes activos en el sistema, junto a la cantidad de agentes atendiendo al mismo
    /// (se guarda en el segundo campo de cada tupla).
    incidents: Arc<Mutex<Vec<(Incident, u8)>>>,

    /// Es la instancia del sistema que controla los Drones del sistema.
    drone_system: DroneSystem,

    /// A traves de este canal el Client va a enviar a la aplicacion los mensajes provenientes del Broker de la red.
    receive_from_client: Arc<Mutex<Receiver<ClientMessage>>>,

    /// Aqui se tienen los Drones activos y su localizacion en el mapa.
    active_drones: Arc<Mutex<HashMap<u32, (Location, Location)>>>,

    updated_drones: Arc<Mutex<VecDeque<(u32, Location, Location)>>>,

    /// Aqui se tienen a todas las camaras del sistema.
    cameras: Arc<Mutex<HashMap<u32, Camera>>>,

    /// Indica si la aplicacion esta conectada o no. La interfaz grafica esta pendiente de los cambios de este flag para
    /// tomar acciones en la aplicacion(como por ejemplo desconectar la aplicacion y volver al menu principal).
    pub connected: Arc<Mutex<bool>>,
}

#[allow(dead_code)]
impl MonitoringApp {
    /// Recibe en el vector de argumentos el username, password, la direccion IP y el port al cual debe conectarse
    /// la aplicacion.
    ///
    /// Se crea el Client de la aplicacion, y tambien se instancia el Sistema Central de Camaras, y el Sistema de Drones.
    pub fn new(args: Vec<String>) -> Result<MonitoringApp, ProtocolError> {
        let address = args[2].to_string() + ":" + &args[3].to_string();

        let camera_system = match CameraSystem::<Client>::with_real_client(address.clone()) {
            Ok(camera_system) => camera_system,
            Err(err) => return Err(err),
        };
        let drone_system =
            DroneSystem::new("src/drones/drone_config.json".to_string(), address.clone());

        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        let monitoring_app_client = MonitoringApp::create_client(
            Some(args[0].to_string()),
            Some(args[1].as_bytes().to_vec()),
            rx,
            address,
            tx2,
            tx.clone(),
        )?;

        Ok(MonitoringApp {
            send_to_client_channel: Arc::new(Mutex::new(tx)),
            incidents: Arc::new(Mutex::new(Vec::new())),
            camera_system: Arc::new(Mutex::new(camera_system)),
            monitoring_app_client,
            drone_system,
            receive_from_client: Arc::new(Mutex::new(rx2)),
            active_drones: Arc::new(Mutex::new(HashMap::new())),
            updated_drones: Arc::new(Mutex::new(VecDeque::new())),
            cameras: Arc::new(Mutex::new(HashMap::new())),
            connected: Arc::new(Mutex::new(true)),
        })
    }

    /// Crea el Client a traves del cual la MonitoringApp va a comunicarse con la red.
    ///
    /// Este Client se va a construir a partir de la configuracion brindada en su archivo de configuracion.
    ///
    /// Una vez conectado, se envian packets que van a suscribir a la aplicacion a sus topics de interes.
    ///
    /// Retorna error en caso de fallar la creacion.
    fn create_client(
        username: Option<String>,
        password: Option<Vec<u8>>,
        receive_from_monitoring_channel: Receiver<Box<dyn MessagesConfig + Send>>,
        address: String,
        send_to_monitoring_channel: Sender<ClientMessage>,
        send_from_monitoring_channel: Sender<Box<dyn MessagesConfig + Send>>,
    ) -> Result<Client, ProtocolError> {
        let mut connect_config =
            client_message::Connect::read_connect_config("src/monitoring/connect_config.json")?;
        if let Some(username) = username {
            connect_config.username = Some(username);
        }

        if let Some(password) = password {
            connect_config.password = Some(password);
        }

        match Client::new(
            receive_from_monitoring_channel,
            address,
            connect_config.clone(),
            send_to_monitoring_channel,
        ) {
            Ok(client) => {
                println!("I'm the MonitoringApp, and my Client is connected successfully!");
                MonitoringApp::subscribe_to_topics(connect_config, send_from_monitoring_channel)?;
                Ok(client)
            }
            Err(_) => Err(ProtocolError::ConectionError),
        }
    }

    /// Contiene las subscripciones a los topics de interes para la MonitoringApp:
    /// necesita la suscripcion al topic de locations de drones("drone_locations"),
    /// al de actualizaciones de camaras("camera_update"), y al topic de
    /// incidentes resueltos("incident_resolved"):
    ///
    /// La idea es que la aplicacion reciba actualizaciones de estado de parte del camera_system,
    /// y de los Drones que tenga creados, y que pueda plasmar estos cambios en la interfaz grafica.
    fn subscribe_to_topics(
        connect_config: Connect,
        send_from_monitoring_channel: Sender<Box<dyn MessagesConfig + Send>>,
    ) -> Result<(), ProtocolError> {
        let subscribe_properties =
            SubscribeProperties::new(1, connect_config.properties.user_properties);
        let topics = vec![
            "drone_locations",
            "camera_update",
            "incident_resolved",
            "incident",
        ];

        for topic_name in topics {
            Self::subscribe_to_topic(
                &topic_name.to_string(),
                &subscribe_properties,
                &connect_config.client_id,
                &send_from_monitoring_channel,
            )?;
        }

        Ok(())
    }

    /// Se encarga de suscribir a la MonitoringApp a un topic determinado.
    fn subscribe_to_topic(
        topic_name: &String,
        subscribe_properties: &SubscribeProperties,
        client_id: &str,
        send_from_monitoring_channel: &Sender<Box<dyn MessagesConfig + Send>>,
    ) -> Result<(), ProtocolError> {
        let client_id_owned = client_id.to_owned();
        let subscribe_config = SubscribeConfig::new(
            topic_name.clone(),
            subscribe_properties.clone(),
            client_id_owned.clone(),
        );

        match send_from_monitoring_channel.send(Box::new(subscribe_config)) {
            Ok(_) => {
                println!(
                    "Monitoring App suscribed to topic {} successfully",
                    topic_name
                );
                Ok(())
            }
            Err(e) => {
                println!("Monitoring: Error sending message: {:?}", e);
                Err(ProtocolError::SubscribeError)
            }
        }
    }

    /// Se encarga de correr al Client de la MonitoringApp, y al Client del CameraSystem, ademas
    /// de manejar los cambios en las distintas entidades de la aplicacion para comunicarselos de forma
    /// periodica a la UI(camaras, drones, incidentes, etc).
    pub fn run_client(&mut self) -> Result<(), ProtocolError> {
        self.handle_update_entities()?;

        println!(";Monitoring App client starts running");
        self.monitoring_app_client.client_run()?;

        println!("Camera System client starts running");
        CameraSystem::<Client>::run_client(None, self.camera_system.clone())?;

        Ok(())
    }

    /// Aqui se manejan los cambios en las entidades del sistema.
    ///
    /// Si update_entities nos retorna false, significa que al Client de la aplicacion ha recibido un packet del tipo disconnect
    /// de parte del Broker, por lo que debe proceder a cerrarse -> el flag de connected que tiene la aplicacion pasa a false,
    /// la UI al captar este cambio, nos envia al menu principal y la aplicacion de monitoreo se cierra.
    fn handle_update_entities(&self) -> Result<(), ProtocolError> {
        let receive_from_client_ref = Arc::clone(&self.receive_from_client);
        let updated_drones_clone = Arc::clone(&self.updated_drones);
        let cameras_clone = Arc::clone(&self.cameras);
        let incidents_clone = Arc::clone(&self.incidents);
        let connected_clone = Arc::clone(&self.connected);

        thread::spawn(move || loop {
            let receiver_clone: Arc<Mutex<Receiver<ClientMessage>>> =
                Arc::clone(&receive_from_client_ref);
            let updated_drones_clone = Arc::clone(&updated_drones_clone);
            let cameras_clone = Arc::clone(&cameras_clone);
            let incidents_clone = Arc::clone(&incidents_clone);
            let connected_clone = Arc::clone(&connected_clone);

            if !update_entities(
                receiver_clone,
                updated_drones_clone,
                cameras_clone,
                incidents_clone,
            ) {
                match connected_clone.lock() {
                    Ok(mut l) => {
                        *l = false;
                        return Ok(());
                    }
                    Err(_) => return Err(ProtocolError::LockError),
                }
            };
        });

        Ok(())
    }

    /// Carga las camaras existentes en el sistema.
    pub fn load_existing_camera_system(&mut self, camera: Camera) -> Result<(), ProtocolError> {
        let mut lock = match self.camera_system.lock() {
            Ok(lock) => lock,
            Err(e) => {
                println!("Monitoring: Error locking camera system: {:?}", e);
                return Err(ProtocolError::LockError);
            }
        };

        match lock.load_existing_camera(camera) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Monitoring: Error loading existing cameras: {:?}", e);
                Err(ProtocolError::CameraError(e.to_string()))
            }
        }
    }

    /// Dada una location para agregar una nueva camara, la agrega al sistema.
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

    /// Publica un nuevo incidente en una location determinada.
    ///
    /// La aplicacion envia un packet del tipo Publish hacia el topic de incidentes, y el sistema
    /// recibe esta notificacion(el sistema central de camaras y cada agente).
    pub fn add_incident(&mut self, location: Location) -> Result<(), ProtocolError> {
        let incident = Incident::new(location);
        let mut incidents = match self.incidents.lock() {
            Ok(incidents) => incidents,
            Err(_) => return Err(ProtocolError::LockError),
        };

        incidents.push((incident.clone(), 0));

        let incident_payload = IncidentPayload::new(incident);
        let publish_config = PublishConfig::read_config(
            "src/monitoring/publish_incident_config.json",
            PayloadTypes::IncidentLocation(incident_payload),
        )?;

        let send_to_client_channel = match self.send_to_client_channel.lock() {
            Ok(send_to_client_channel) => send_to_client_channel,
            Err(_) => return Err(ProtocolError::LockError),
        };

        match send_to_client_channel.send(Box::new(publish_config)) {
            Ok(_) => {
                println!("Incidente published successfully");
                Ok(())
            }
            Err(e) => {
                println!("Error publishing incident {}", e);
                Err(ProtocolError::PublishError)
            }
        }
    }

    /// Agrega un Drone al sistema.
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

    pub fn load_existing_drone(
        &mut self,
        location: Location,
        drone_center_id: u32,
    ) -> Result<u32, ProtocolError> {
        match self
            .drone_system
            .load_existing_drone(location, drone_center_id)
        {
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

    /// Carga un centro de Drones existente.
    pub fn load_existing_drone_center(&mut self, location: Location) -> u32 {
        self.drone_system
            .load_existing_drone_center(location)
            .map_or(0, |id| id)
    }

    /// Retorna los incidentes activos en el sistema.
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

    pub fn get_updated_drones(&self) -> Arc<Mutex<VecDeque<(u32, Location, Location)>>> {
        // match self.updated_drones.lock() {
        //     Ok(updated_drones) => updated_drones.clone(),
        //     Err(_) => VecDeque::new(),
        // }
        self.updated_drones.clone()
    }

    /// Retorna las camaras activas en el sistema.
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
                println!("Error disconnecting the Monitoring App");
                return Err(ProtocolError::DisconnectError);
            }
        }

        let system = match self.camera_system.lock() {
            Ok(system) => system,
            Err(_) => return Err(ProtocolError::LockError),
        };

        system.disconnect()?;
        self.drone_system.disconnect_system()?;

        println!("Monitoring App disconnected successfully");

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
/// Si se recibe un mensaje del topic attending_incident, se lleva a cabo un procedimiento:
/// - La Monitoring App lleva un registro de todos los incidentes ingresados, junto a la cantidad de Drones que
///   estan resolviendolo(apenas se publica un incidente nuevo, este valor sera 0).
/// - A medida que vaya recibiendo mensajes sobre ese topic, se ira incrementando el valor de este contador.
/// - Si este contador llega a 2, se publica un mensaje con topic incident_resolved, el cual recibiran los Drones,
///   y aquellos que no fueron a resolver el incidente dejaran de ir a resolverlo, y volveran a patrullar en su area.
pub fn update_entities(
    recieve_from_client: Arc<Mutex<Receiver<ClientMessage>>>,
    updated_drones: Arc<Mutex<VecDeque<(u32, Location, Location)>>>,
    cameras: Arc<Mutex<HashMap<u32, Camera>>>,
    incidents: Arc<Mutex<Vec<(Incident, u8)>>>,
) -> bool {
    let receiver = match recieve_from_client.lock() {
        Ok(receiver) => receiver,
        Err(_) => return true,
    };

    match receiver.try_recv() {
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
                match topic_name.as_str() {
                    "drone_locations" => {
                        println!("Monitoring: Drone location received");
                        // let mut active_drones: std::sync::MutexGuard<
                        //     HashMap<u32, (Location, Location)>,
                        // > = match active_drones.try_lock() {
                        //     Ok(active_drones) => active_drones,
                        //     Err(_) => return true,
                        // };
                        let mut updated_drones: std::sync::MutexGuard<
                            VecDeque<(u32, Location, Location)>,
                        > = match updated_drones.try_lock() {
                            Ok(updated_drones) => updated_drones,
                            Err(_) => return true,
                        };
                        if let PayloadTypes::DroneLocation(id, drone_locationn, target_location) =
                            payload
                        {
                            // active_drones.insert(id, (drone_locationn, target_location));
                            updated_drones.push_back((id, drone_locationn, target_location));
                        }
                    }
                    "camera_update" => {
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
                    }
                    "incident_resolved" => {
                        if let PayloadTypes::IncidentLocation(incident_payload) = payload {
                            let mut incidents = match incidents.lock() {
                                Ok(incidents) => incidents,
                                Err(_) => return true,
                            };
                            let mut to_remove = Vec::new();

                            for (incident, _count) in incidents.iter_mut() {
                                if incident.get_location()
                                    == incident_payload.get_incident().get_location()
                                {
                                    to_remove.push(incident.clone());
                                }
                            }

                            incidents.retain(|(inc, _)| !to_remove.contains(inc));
                        }
                    }
                    "incident" => {
                        if let PayloadTypes::IncidentLocation(incident_payload) = payload {
                            let incident = incident_payload.get_incident();
                            let mut incidents = match incidents.lock() {
                                Ok(incidents) => incidents,
                                Err(_) => return true,
                            };

                            if !incidents
                                .iter()
                                .any(|(existing_incident, _)| *existing_incident == incident)
                            {
                                incidents.push((incident, 0));
                            }
                        }
                    }
                    _ => {}
                }
                true
            }
            ClientMessage::Disconnect {
                reason_code: _,
                session_expiry_interval: _,
                reason_string: _,
                client_id: _,
            } => false,
            _ => true,
        },
        Err(_) => true,
    }
}
