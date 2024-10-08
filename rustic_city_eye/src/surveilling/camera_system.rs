use std::{
    collections::HashMap,
    path::Path,
    sync::{
        mpsc::{self, channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use rand::Rng;

use crate::{
    monitoring::{incident::Incident, persistence::Persistence},
    mqtt::{
        client::{Client, ClientTrait},
        client_message::{self, ClientMessage},
        disconnect_config::DisconnectConfig,
        messages_config::MessagesConfig,
        protocol_error::ProtocolError,
        publish::publish_config::PublishConfig,
        subscribe_config::SubscribeConfig,
        subscribe_properties::SubscribeProperties,
    },
    surveilling::camera::Camera,
    utils::{
        incident_payload::IncidentPayload, location::Location, payload_types::PayloadTypes,
        threadpool::ThreadPool, watcher::watch_directory,
    },
};

const AREA_DE_ALCANCE: f64 = 0.0025;
const NIVEL_DE_PROXIMIDAD_MAXIMO: f64 = AREA_DE_ALCANCE;
const PATH: &str = "src/surveilling/cameras.";
const TIME_INTERVAL_IN_SECS: u64 = 1;
const PATH_POSITION: usize = 1;

use super::camera_error::CameraError;
#[derive(Debug)]

/// Entidad encargada de gestionar todas las camaras. Tiene como parametro a su instancia de cliente, utiliza un hash `<ID, Camera>` como estructura principal y diferentes channels para comunicarse con su cliente.
/// Los mensajes recibidos le llegan mediante el channel `reciev_from_client` y envia una config con los mensajes que quiere enviar mediante `send_to_client_channel``
pub struct CameraSystem<T: ClientTrait + Clone + Send + Sync> {
    pub send_to_client_channel: Arc<Mutex<Sender<Box<dyn MessagesConfig + Send>>>>,
    camera_system_client: T,
    cameras: Arc<Mutex<HashMap<u32, Camera>>>,
    reciev_from_client: Arc<Mutex<Receiver<client_message::ClientMessage>>>,
    snapshot: Vec<Camera>,
}

impl<T: ClientTrait + Clone + Send + Sync> Clone for CameraSystem<T> {
    fn clone(&self) -> Self {
        CameraSystem {
            send_to_client_channel: Arc::clone(&self.send_to_client_channel),
            camera_system_client: self.camera_system_client.clone(),
            cameras: self.cameras.clone(),
            reciev_from_client: Arc::clone(&self.reciev_from_client),
            snapshot: self.snapshot.clone(),
        }
    }
}

impl<T: ClientTrait + Clone + Send + Sync + 'static> CameraSystem<T> {
    /// Crea un nuevo camera system con un cliente de mqtt
    ///
    /// Envia un connect segun la configuracion del archivo connect_config.json
    ///
    /// Se subscribe a los mensajes de tipo "incident" e "incident_resolved"
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
        let client_id = camera_system_client.get_client_id();
        let subscribe_config = SubscribeConfig::new(
            "incident".to_string(),
            SubscribeProperties::new(1, vec![]),
            client_id.clone(),
        );

        let _ = tx.send(Box::new(subscribe_config));
        let subscribe_config = SubscribeConfig::new(
            "incident_resolved".to_string(),
            SubscribeProperties::new(1, vec![]),
            client_id.clone(),
        );

        let _ = tx.send(Box::new(subscribe_config));

        let subscribe_config = SubscribeConfig::new(
            "single_camera_disconnect".to_string(),
            SubscribeProperties::new(1, vec![]),
            client_id,
        );

        let _ = tx.send(Box::new(subscribe_config));

        Ok(CameraSystem {
            send_to_client_channel: Arc::new(Mutex::new(tx)),
            camera_system_client,
            cameras: Arc::new(Mutex::new(HashMap::new())),
            reciev_from_client: Arc::new(Mutex::new(rx2)),
            snapshot: Vec::new(),
        })
    }

    /// Bloquea el mutex de las camaras y devuelve un guard
    fn lock_cameras(
        &self,
    ) -> Result<std::sync::MutexGuard<std::collections::HashMap<u32, Camera>>, CameraError> {
        match self.cameras.lock() {
            Ok(guard) => Ok(guard),
            Err(_) => Err(CameraError::ArcMutexError(
                "Error locking cameras mutex".to_string(),
            )),
        }
    }

    /// Carga una camara existente en el sistema
    pub fn load_existing_camera(&mut self, camera: Camera) -> Result<(), CameraError> {
        let mut cameras = self.lock_cameras()?;
        cameras.insert(camera.get_id(), camera);
        Ok(())
    }

    /// Agrega una camara al sistema
    pub fn add_camera(&mut self, location: Location) -> Result<u32, CameraError> {
        let mut rng = rand::thread_rng();
        let mut id = rng.gen();

        let mut cameras = self.lock_cameras()?;
        while cameras.contains_key(&id) {
            id = rng.gen();
        }

        let camera = Camera::new(location, id)?;
        let _ = Persistence::add_camera_to_file(camera.clone());
        println!("Camera System creates camera with id: {:?}", id);
        cameras.insert(id, camera);

        Ok(id)
    }

    pub fn get_cameras(&self) -> Arc<Mutex<HashMap<u32, Camera>>> {
        self.cameras.clone()
    }

    pub fn get_camera_by_id(&mut self, id: u32) -> Option<Camera> {
        let cameras = match self.cameras.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return None;
            }
        };
        let camera = cameras.get(&id).cloned();
        camera
    }

    pub fn get_camera(&mut self) -> Option<Camera> {
        let cameras = match self.cameras.lock() {
            Ok(guard) => guard,
            Err(_) => return None,
        };
        let keys: Vec<&u32> = cameras.keys().collect();
        if keys.is_empty() {
            None
        } else {
            let idx = rand::thread_rng().gen_range(0..keys.len());
            cameras.get(keys[idx]).cloned()
        }
    }

    /// recibe los diferentes mensajes reenviados por el cliente.
    ///
    /// Si el mensaje es un publish con topic accidente, activa las camaras cercanas a la location del incidente.
    ///
    /// Si el mensaje es un publish con topic accidenteresuelto, desactiva las camaras cercanas a la location del incidente.
    ///
    /// Si el mensaje es un publish con topic single_camera_disconnect y el ID enviado en el payload coincide con el de una camara, la elimina del sistema.
    ///
    /// Recibe un reciever opcional para poder testear la funcion, si este es None, utiliza el propio del broker
    ///
    pub fn run_client(
        parameter_reciever: Option<Arc<Mutex<Receiver<ClientMessage>>>>,
        system: Arc<Mutex<CameraSystem<Client>>>,
    ) -> Result<(), ProtocolError> {
        let system_clone_one = Arc::clone(&system);
        let system_clone_two = Arc::clone(&system);

        thread::spawn(move || {
            let mut lock = match system_clone_one.lock() {
                Ok(guard) => guard,
                Err(_) => {
                    return;
                }
            };
            match lock.camera_system_client.client_run() {
                Ok(_) => {}
                Err(e) => {
                    println!("CameraSys: Error running client: {:?}", e);
                }
            }
        });

        thread::spawn(move || {
            handle_incident_messages(system_clone_two, parameter_reciever);
        });

        thread::spawn(move || watch_dirs(system));
        Ok(())
    }

    /// Busca dentro de un path relativo hacia el directorio donde se guardan las imagenes de una camara determinada
    /// el direntry del mismo.
    fn get_relative_path_to_camera(path: &str) -> Option<&str> {
        let parts: Vec<&str> = path.split('/').collect();

        if let Some(pos) = parts.iter().position(|&x| x == "cameras.") {
            if pos + 1 < parts.len() {
                return Some(parts[pos + 1]);
            }
        }
        None
    }

    /// Desconecta el sistema de camaras del broker
    pub fn disconnect(&self) -> Result<(), ProtocolError> {
        let disconnect_config = DisconnectConfig::new(
            0x00_u8,
            1,
            "normal".to_string(),
            self.camera_system_client.get_client_id(),
        );
        let send_to_client_channel = match self.send_to_client_channel.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return Err(ProtocolError::SendError(
                    "Error locking send_to_client_channel".to_string(),
                ));
            }
        };
        match send_to_client_channel.send(Box::new(disconnect_config)) {
            Ok(_) => {}
            Err(e) => {
                return Err(ProtocolError::SendError(e.to_string()));
            }
        }

        println!("The camera system has been disconnected");

        Ok(())
    }

    /// Recibe una location y activas todas las camaras que esten a menos de AREA_DE_ALCANCE de esta.
    ///
    /// Al activarlas se las pasa de modo ahorro de energia a modo activo
    ///
    /// Adicionalmente, cada camara activada despues avisa a las camaras colindantes para que,
    /// si estan a la distancia requerida, tambien se activen.
    pub fn activate_cameras(&mut self, location: Location) -> Result<(), CameraError> {
        let locations_to_activate: Vec<Location> = {
            let mut cameras = match self.cameras.lock() {
                Ok(guard) => guard,
                Err(_) => {
                    return Err(CameraError::ArcMutexError(
                        "Error locking cameras mutex".to_string(),
                    ));
                }
            };

            cameras
                .values_mut()
                .filter_map(|camera| {
                    let distancia = camera.get_location().distance(location);
                    if distancia <= AREA_DE_ALCANCE {
                        camera.set_sleep_mode(false);
                        Some(camera.get_location())
                    } else {
                        None
                    }
                })
                .collect()
        };
        for loc in locations_to_activate {
            self.activate_cameras_by_camera_location(loc)?;
        }
        println!("Camera System: cameras activated");
        self.publish_cameras_update()?;

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
        let mut locations_to_activate = Vec::new();
        {
            let mut cameras = match self.cameras.lock() {
                Ok(guard) => guard,
                Err(_) => {
                    return Err(CameraError::ArcMutexError(
                        "Error locking cameras mutex".to_string(),
                    ));
                }
            };
            for camera in cameras.values_mut() {
                let distancia = camera.get_location().distance(location);
                if distancia <= NIVEL_DE_PROXIMIDAD_MAXIMO && camera.get_sleep_mode() {
                    camera.set_sleep_mode(false);
                    locations_to_activate.push(camera.get_location());
                }
            }
        }

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
        let locations_to_activate: Vec<Location> = {
            let mut cameras = match self.cameras.lock() {
                Ok(guard) => guard,
                Err(_) => {
                    return Err(CameraError::ArcMutexError(
                        "Error locking cameras mutex".to_string(),
                    ));
                }
            };
            cameras
                .values_mut()
                .filter_map(|camera| {
                    let distancia = camera.get_location().distance(location);
                    if distancia <= AREA_DE_ALCANCE {
                        camera.set_sleep_mode(true);
                        Some(camera.get_location())
                    } else {
                        None
                    }
                })
                .collect()
        };
        for loc in locations_to_activate {
            self.deactivate_cameras_by_camera_location(loc)?;
        }

        self.publish_cameras_update()?;

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
        let mut locations_to_activate = Vec::new();
        {
            let mut cameras = match self.cameras.lock() {
                Ok(guard) => guard,
                Err(_) => {
                    return Err(CameraError::ArcMutexError(
                        "Error locking cameras mutex".to_string(),
                    ));
                }
            };
            for camera in cameras.values_mut() {
                let distancia = camera.get_location().distance(location);
                if distancia <= NIVEL_DE_PROXIMIDAD_MAXIMO && !camera.get_sleep_mode() {
                    camera.set_sleep_mode(true);
                    locations_to_activate.push(camera.get_location());
                }
            }
        }

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
        let packet_id = self.camera_system_client.assign_packet_id();
        let message = message.parse_message(packet_id);
        let lock = match self.send_to_client_channel.lock() {
            Ok(lock) => lock,
            Err(_) => {
                return Err(CameraError::SendError);
            }
        };
        match lock.send(Box::new(message)) {
            Ok(_) => {}
            Err(e) => {
                println!("CameraSys: Error sending message: {:?}", e);
                return Err(CameraError::SendError);
            }
        };
        Ok(())
    }

    /// Compara la snapshot anterior que tiene almacenada con la actual, envia mediante un pub
    /// Las camaras que hayan cambiado de estado entre el ultimo publish y este.
    /// En un vector de camaras.
    /// Si no hay cambios, no envia nada
    /// Almacena la nueva snapshot para la proxima comparacion
    pub fn publish_cameras_update(&mut self) -> Result<(), CameraError> {
        let mut new_snapshot: Vec<Camera> = Vec::new();
        let cameras = match self.cameras.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return Err(CameraError::ArcMutexError(
                    "Error locking cameras mutex".to_string(),
                ));
            }
        };
        for camera in cameras.values() {
            new_snapshot.push(camera.clone());
        }

        let mut updated_cameras = Vec::new();
        for camera in new_snapshot.iter() {
            if (!self.snapshot.contains(&camera.clone()))
                || (self.snapshot.contains(&camera.clone())
                    && camera.get_sleep_mode()
                        != match self.snapshot.iter().find(|&x| x == camera) {
                            Some(camera) => camera.get_sleep_mode(),
                            None => false,
                        })
            {
                updated_cameras.push(camera.clone());
            }
        }
        if updated_cameras.is_empty() {
            return Ok(());
        }
        println!("Camera System: publishing camera update to broker");

        let publish_config = match PublishConfig::read_config(
            "./src/surveilling/publish_config_update.json",
            PayloadTypes::CamerasUpdatePayload(updated_cameras),
        ) {
            Ok(config) => config,
            Err(_) => {
                return Err(CameraError::SendError);
            }
        };

        let lock = match self.send_to_client_channel.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return Err(CameraError::SendError);
            }
        };

        match lock.send(Box::new(publish_config)) {
            Ok(_) => {}
            Err(_) => return Err(CameraError::SendError),
        }
        self.snapshot = new_snapshot;

        Ok(())
    }
}

/// Se encarga de supervisar el directorio de camaras.
/// Cada vez que se detecta un cambio en el directorio, se revisa el hash para saber si ese evento fue procesado recientemente.
/// (eso es para evitar que por algun bug un evento se detecte +1 vez, como copiados y pegados que a veces involucran mas de un evento)
/// De no ser asi, se lo procesa
fn watch_dirs(system: Arc<Mutex<CameraSystem<Client>>>) -> Result<(), ProtocolError> {
    let pool = ThreadPool::new(10);
    let (tx, rx) = channel();
    watch_directory(Path::new(PATH).to_path_buf(), tx);
    let mut last_event_times: HashMap<String, Instant> = HashMap::new();

    loop {
        match rx.recv() {
            Ok(event) => {
                let now = Instant::now();
                let should_process = match last_event_times.get(&event[PATH_POSITION]) {
                    Some(&last_time) => {
                        now.duration_since(last_time) > Duration::from_secs(TIME_INTERVAL_IN_SECS)
                    }
                    None => true,
                };

                if should_process {
                    if let Some(value) = process_dir_change(
                        &mut last_event_times,
                        &event[PATH_POSITION],
                        now,
                        event.clone(),
                        &system,
                        &pool,
                    ) {
                        return value;
                    }
                }
            }
            Err(e) => {
                println!("watch error: {:?}", e);
                break;
            }
        }
    }
    Ok(())
}

/// Maneja los mensajes relacionados con incidentes que se reciben del cliente,
/// activando o desactivando cámaras según la ubicación del incidente.
///
/// # Parámetros
///
/// - `system`: Un `Arc<Mutex<CameraSystem<Client>>>` que contiene el sistema de cámaras.
/// - `parameter_reciever`: Una `Option<Arc<Mutex<Receiver<ClientMessage>>>>` que puede
///   contener un receptor de mensajes del cliente. Si es `None`, se obtiene de
///   `system.reciev_from_client`.
///
/// # Comportamiento
///
/// La función entra en un bucle infinito donde:
///
/// 1. Obtiene una copia del sistema de cámaras.
/// 2. Si hay una ubicación de incidente pendiente (`incident_location`), intenta activar
///    las cámaras en esa ubicación y luego resetea `incident_location`.
/// 3. Si hay una ubicación de incidente resuelto pendiente (`solved_incident_location`),
///    intenta desactivar las cámaras en esa ubicación y luego resetea
///    `solved_incident_location`.
/// 4. Bloquea el receptor para recibir un mensaje del cliente. Si el mensaje indica
///    que un incidente ha ocurrido o ha sido resuelto, actualiza las ubicaciones
///    correspondientes (`incident_location` o `solved_incident_location`).
/// 5. Libera el bloqueo del receptor antes de continuar el ciclo para evitar
///    deadlocks.
fn handle_incident_messages(
    system: Arc<Mutex<CameraSystem<Client>>>,
    parameter_reciever: Option<Arc<Mutex<Receiver<ClientMessage>>>>,
) {
    let mut incident_location: Option<Location> = None;
    let mut solved_incident_location: Option<Location> = None;
    let reciever: Arc<Mutex<Receiver<ClientMessage>>> = match parameter_reciever {
        Some(parameter_reciever) => parameter_reciever,
        None => match system.lock() {
            Ok(guard) => guard.reciev_from_client.clone(),
            Err(_) => {
                return;
            }
        },
    };

    loop {
        let system_clone = Arc::clone(&system);

        let mut lock = match system_clone.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => {
                return;
            }
        };

        if let Some(location) = incident_location {
            if let Err(e) = lock
                .activate_cameras(location)
                .map_err(|e| ProtocolError::CameraError(e.to_string()))
            {
                println!("CameraSys: Error activating cameras: {:?}", e);
            }
            incident_location = None;
        }
        if let Some(location) = solved_incident_location {
            if let Err(e) = lock
                .deactivate_cameras(location)
                .map_err(|e| ProtocolError::CameraError(e.to_string()))
            {
                println!("CameraSys: Error deactivating cameras: {:?}", e);
            }
            solved_incident_location = None;
        }

        let reciever = match reciever.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return;
            }
        };

        match reciever.recv() {
            Ok(message) => {
                process_client_message(
                    message,
                    &mut incident_location,
                    &mut solved_incident_location,
                );
                drop(reciever); // Libera el lock acá
                continue;
            }
            Err(_) => {
                continue;
            }
        }
    }
}
/// Se encarga de procesar los mensajes del cliente.
/// Si el mensaje es un publish con topic incidente, guarda la location del incidente.
/// Si el mensaje es un publish con topic incidente resuelto, guarda la location del incidente resuelto.

fn process_client_message(
    message: ClientMessage,
    incident_location: &mut Option<Location>,
    solved_incident_location: &mut Option<Location>,
) {
    match message {
        client_message::ClientMessage::Publish {
            topic_name,
            payload: PayloadTypes::IncidentLocation(payload),
            ..
        } => {
            if topic_name == "incident" {
                *incident_location = Some(payload.get_incident().get_location());
            } else if topic_name == "incident_resolved" {
                *solved_incident_location = Some(payload.get_incident().get_location());
            }
        }
        _ => {
            // Maneja otros tipos de mensajes si es necesario
        }
    }
}

/// Procesa cualquier evento de cambio en en el directorio observado
/// Si es un evento del tipo Create (creacion de directorio), notifica mediante logging al usuario
///
/// Si el evento es de tipo modify (por ejemplo, un archivo ya existente es movido al dir en cuestion), y es una imagen, la camara correspondiente a la imagen analiza la imagen. Si es un incidente, se envia un mensaje al broker
fn process_dir_change(
    last_event_times: &mut HashMap<String, Instant>,
    str_path: &str,
    now: Instant,
    event: Vec<String>,
    system: &Arc<Mutex<CameraSystem<Client>>>,
    pool: &ThreadPool,
) -> Option<Result<(), ProtocolError>> {
    last_event_times.insert(str_path.to_string().clone(), now);
    match event[0].as_str() {
        "Error" => {
            return Some(Err(ProtocolError::WatcherError(event[1].clone())));
        }
        "New file detected" => {
            if (str_path.ends_with(".jpg") || str_path.ends_with(".jpeg"))
                || str_path.ends_with(".png")
            {
                analize_image(event, system, pool);
            }
        }
        "New directory detected" => {
            let camera_id = match CameraSystem::<Client>::get_relative_path_to_camera(str_path) {
                Some(id) => id,
                None => {
                    return Some(Err(ProtocolError::CameraError(
                        "Error parsing the camera id".to_string(),
                    )))
                }
            };

            println!("Camera's ID directory has been created {:?}", camera_id);
        }
        _ => {}
    }
    None
}

/// Al detectar que al directorio de una camara se le agrego una imagen, se busca esa camara dentro del sistema
/// central de camaras, y ella se encarga de clasificar la imagen.
/// Si este devuelve true, la imagen corresponde a un incidente y se envia el respectivo mensaje al broker
/// Con la location de este incidente siendo la de la cámara.
fn analize_image(event: Vec<String>, system: &Arc<Mutex<CameraSystem<Client>>>, pool: &ThreadPool) {
    let system_clone = Arc::clone(system);
    pool.execute(move || -> Result<(), ProtocolError> {
        let system_clone2 = Arc::clone(&system_clone);
        let str_path = event[1].clone();

        let camera_id = match CameraSystem::<Client>::get_relative_path_to_camera(&str_path) {
            Some(id) => id,
            None => {
                return Err(ProtocolError::CameraError(
                    "Error while parsing the camera's id".to_string(),
                ))
            }
        };

        let camera_id = match camera_id.parse::<u32>() {
            Ok(camera_id) => camera_id,
            Err(_) => {
                println!("Error while parsing the camera's id");
                return Err(ProtocolError::InvalidCommand(
                    "Invalid camera id".to_string(),
                ));
            }
        };
        println!("Camera id {:?} is analyzing an image", camera_id);
        let camera = match system_clone.lock() {
            Ok(mut system) => match system.get_camera_by_id(camera_id) {
                Some(camera) => camera,
                None => {
                    return Err(ProtocolError::InvalidCommand(
                        "Camera not found".to_string(),
                    ));
                }
            },
            Err(_) => {
                return Err(ProtocolError::ArcMutexError(
                    "Error locking cameras mutex".to_string(),
                ));
            }
        };

        let classification_result = camera.annotate_image(&str_path)?;

        if !classification_result {
            println!("Not an incident");
        } else {
            publish_incident(&system_clone2, camera)?;
        }
        Ok(())
    });
}

/// Publica el incidente que la camara detecto.
fn publish_incident(
    camera_system_ref: &Arc<Mutex<CameraSystem<Client>>>,
    camera: Camera,
) -> Result<(), ProtocolError> {
    let location = camera.get_location();
    let incident = Incident::new(location);
    let incident_payload = IncidentPayload::new(incident);
    let publish_config = PublishConfig::read_config(
        "./src/surveilling/publish_incident_config.json",
        PayloadTypes::IncidentLocation(incident_payload),
    )
    .map_err(|e| ProtocolError::SendError(e.to_string()))?;
    let mut lock = match camera_system_ref.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return Err(ProtocolError::ArcMutexError(
                "Error locking cameras mutex".to_string(),
            ));
        }
    };
    match lock.send_message(Box::new(publish_config)) {
        Ok(_) => Ok(()),
        Err(e) => Err(ProtocolError::SendError(e.to_string())),
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
    use std::fs::File;
    use std::path::Path;
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;

    use super::*;
    use crate::monitoring::incident::Incident;
    use crate::mqtt::broker::Broker;
    use crate::mqtt::client_message::ClientMessage;
    use crate::utils::incident_payload::IncidentPayload;
    use std::time::Duration;
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
            let id = camera_system.add_camera(location).unwrap();
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
            let id = camera_system.add_camera(location).unwrap();
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
            let id = camera_system.add_camera(location).unwrap();
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
            let id3: u32 = camera_system.add_camera(location3).unwrap();
            let location4 = Location::new(10.0, 20.0);
            let id4 = camera_system.add_camera(location4).unwrap();
            let location5 = Location::new(0.0001, 0.0002);
            let id5 = camera_system.add_camera(location5).unwrap();
            assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 5);
            assert_eq!(
                camera_system.get_camera_by_id(id).unwrap().get_location(),
                location
            );
            let incident_location = Location::new(0.0, 0.0);
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
            let location = Location::new(10.0 / 10000.0, 0.0);
            let id = camera_system.add_camera(location).unwrap();
            let location2 = Location::new(11.0 / 10000.0, 0.0);
            let id2 = camera_system.add_camera(location2).unwrap();

            assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 2);

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
            let id = camera_system.add_camera(location).unwrap();
            assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 1);
            assert_eq!(
                camera_system.get_camera_by_id(id).unwrap().get_location(),
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
                camera_system.get_camera_by_id(id).unwrap().get_location(),
                location
            );
            let incident_location = Location::new(0.0, 0.0);
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
            let location = Location::new(10.0 / 10000.0, 0.0);
            let id = camera_system.add_camera(location).unwrap();
            let location2 = Location::new(11.0 / 10000.0, 0.0);
            let id2 = camera_system.add_camera(location2).unwrap();

            assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 2);

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
            assert!(!camera_system.get_camera_by_id(id).unwrap().get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id2)
                .unwrap()
                .get_sleep_mode());

            let incident_location = Location::new(0.0, 0.0);
            camera_system.activate_cameras(incident_location).unwrap();
            assert!(!camera_system.get_camera_by_id(id).unwrap().get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id2)
                .unwrap()
                .get_sleep_mode());

            let incident_location = Location::new(2.00001, 2.00001);
            let id3 = camera_system.add_camera(incident_location).unwrap();
            let id4 = camera_system.add_camera(incident_location).unwrap();
            camera_system.activate_cameras(incident_location).unwrap();
            assert!(!camera_system.get_camera_by_id(id).unwrap().get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id2)
                .unwrap()
                .get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id3)
                .unwrap()
                .get_sleep_mode());
            assert!(!camera_system
                .get_camera_by_id(id4)
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
                camera_system.get_camera_by_id(id).unwrap().get_location(),
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

    #[test]

    fn test_13_creo_dirs_y_al_hacer_disconnect_se_borran() {
        let args = vec!["127.0.0.1".to_string(), "6000".to_string()];
        let addr = "127.0.0.1:6000";
        let mut broker = match Broker::new(args) {
            Ok(broker) => broker,
            Err(e) => {
                panic!("Error creating broker: {:?}", e)
            }
        };
        thread::spawn(move || {
            _ = broker.server_run();
        });

        let handle = thread::spawn(move || {
            let mut camera_system =
                CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
            let location = Location::new(1.0, 2.0);
            let id: u32 = camera_system.add_camera(location).unwrap();
            let location2 = Location::new(1.0, 5.0);
            let id2 = camera_system.add_camera(location2).unwrap();
            assert_eq!(camera_system.get_cameras().lock().unwrap().len(), 2);
            assert_eq!(
                camera_system.get_camera_by_id(id).unwrap().get_location(),
                location
            );
            assert_eq!(
                camera_system.get_camera_by_id(id2).unwrap().get_location(),
                location2
            );

            let dir_name = format!("./{}", id);
            let path1 = "src/surveilling/cameras".to_string() + &dir_name;
            assert!(Path::new(path1.as_str()).exists());

            let dir_name = format!("./{}", id2);
            let path2 = "src/surveilling/cameras".to_string() + &dir_name;
            assert!(Path::new(path2.as_str()).exists());

            camera_system.disconnect().unwrap();

            assert!(!Path::new(path1.as_str()).exists());
            assert!(!Path::new(path2.as_str()).exists());
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_creo_file_en_dir_e_imprime_ok() {
        let args = vec!["127.0.0.1".to_string(), "5055".to_string()];
        let addr = "127.0.0.1:5055";
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
        let locurote = thread::spawn(move || {
            let camera_system = CameraSystem::<Client>::with_real_client(addr.to_string()).unwrap();
            let camera_arc = Arc::new(Mutex::new(camera_system));
            let can_start_second_thread = Arc::new((Mutex::new(false), Condvar::new()));
            let second_thread_completed = Arc::new((Mutex::new(false), Condvar::new()));
            let camera_id_shared = Arc::new(Mutex::new(None)); // Shared camera ID

            let (
                can_start_second_thread_clone,
                second_thread_completed_clone,
                camera_id_shared_clone,
            ) = (
                Arc::clone(&can_start_second_thread),
                Arc::clone(&second_thread_completed),
                Arc::clone(&camera_id_shared),
            );

            // Thread 1: Camera system run
            let camera_arc_clone_for_thread1 = Arc::clone(&camera_arc);
            let camera_arc_clone_for_thread2 = Arc::clone(&camera_arc);
            let handler_camera_system = thread::spawn(move || {
                CameraSystem::<Client>::run_client(None, camera_arc_clone_for_thread1).unwrap();
                let mut camera_system = camera_arc_clone_for_thread2.lock().unwrap();
                let location = Location::new(1.0, 2.0);
                let id: u32 = camera_system.add_camera(location).unwrap();
                *camera_id_shared.lock().unwrap() = Some(id);
                // Signal to start the second thread
                {
                    let (lock, cvar) = &*can_start_second_thread_clone;
                    let mut start = lock.lock().unwrap();
                    *start = true;
                    cvar.notify_one();
                }
                // Wait for the second thread to complete
                {
                    let (lock, cvar) = &*second_thread_completed_clone;
                    let mut completed = lock.lock().unwrap();
                    while !*completed {
                        completed = cvar.wait(completed).unwrap();
                    }
                }
                camera_system.disconnect().unwrap();
            });

            // Thread 2: File creation and deletion
            let handler_file_operations = thread::spawn(move || {
                // Wait for the signal from the first thread to start
                {
                    let (lock, cvar) = &*can_start_second_thread;
                    let mut start = lock.lock().unwrap();
                    while !*start {
                        start = cvar.wait(start).unwrap();
                    }
                } // Retrieve the shared camera ID
                let camera_id = camera_id_shared_clone.lock().unwrap();
                let path1 =
                    "src/surveilling/cameras./".to_string() + &camera_id.unwrap().to_string();
                let dir_path = Path::new(path1.as_str());
                let temp_file_path = dir_path.join("temp_file.txt");
                File::create(&temp_file_path).expect("Failed to create temporary file");

                // Wait 2 seconds
                thread::sleep(Duration::from_secs(2));

                std::fs::remove_file(temp_file_path).expect("Failed to remove temporary file");
                {
                    let (lock, cvar) = &*second_thread_completed;
                    let mut completed = lock.lock().unwrap();
                    *completed = true;
                    cvar.notify_one();
                }
            });

            // Wait for both threads to complete
            handler_camera_system.join().unwrap();
            handler_file_operations.join().unwrap();
        });
        locurote.join().unwrap();
    }
}
