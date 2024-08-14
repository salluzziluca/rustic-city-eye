use client::client::{Client, ClientTrait};
use protocol::{client_message::{self, ClientMessage}, disconnect::disconnect_config::DisconnectConfig, messages_config::MessagesConfig, publish::{payload_types::PayloadTypes, publish_config::PublishConfig}, subscribe::{subscribe_config::SubscribeConfig, subscribe_properties::SubscribeProperties}};
use rand::Rng;
use utils::{camera::Camera, camera_error::CameraError, incident::Incident, incident_payload::IncidentPayload, location::Location, protocol_error::ProtocolError, threadpool::ThreadPool, watcher::watch_directory};

use std::{
    collections::HashMap, path::Path, sync::{
        mpsc::{self, channel, Receiver, Sender},
        Arc, Mutex,
    }, thread, time::{Duration, Instant}
};

const AREA_DE_ALCANCE: f64 = 0.0025;
const NIVEL_DE_PROXIMIDAD_MAXIMO: f64 = AREA_DE_ALCANCE;
const PATH: &str = "./camera_system/cameras.";
const TIME_INTERVAL_IN_SECS: u64 = 1;
const PATH_POSITION: usize = 1;

#[derive(Debug)]

/// Entidad encargada de gestionar todas las camaras. Tiene como parametro a su instancia de cliente, utiliza un hash `<ID, Camera>` como estructura principal y diferentes channels para comunicarse con su cliente.
/// Los mensajes recibidos le llegan mediante el channel `reciev_from_client` y envia una config con los mensajes que quiere enviar mediante `send_to_client_channel``
pub struct CameraSystem<T: ClientTrait + Clone + Send + Sync> {
    pub send_to_client_channel: Arc<Mutex<Sender<Box<dyn MessagesConfig + Send>>>>,
    pub camera_system_client: T,
    cameras: Arc<Mutex<HashMap<u32, Camera>>>,
    reciev_from_client: Arc<Mutex<Receiver<ClientMessage>>>,
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
            client_message::Connect::read_connect_config("camera_system/packets_config/connect_config.json")?;

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
    pub fn add_camera(&mut self, location: Location) -> Result<(u32, Camera), CameraError> {
        let mut rng = rand::thread_rng();
        let mut id = rng.gen();

        let mut cameras = self.lock_cameras()?;
        while cameras.contains_key(&id) {
            id = rng.gen();
        }

        let camera = Camera::new(location, id)?;
        println!("Camera System creates camera with id: {:?}", id);
        cameras.insert(id, camera.clone());

        Ok((id, camera))
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
    /// Recibe un reciever opcional para poder testear la funcion, si este es None, utiliza el propio del broker
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
            let mut incident_location: Option<Location> = None;
            let mut solved_incident_location: Option<Location> = None;
            let reciever: Arc<Mutex<Receiver<ClientMessage>>> = match parameter_reciever {
                Some(parameter_reciever) => parameter_reciever,
                None => match system_clone_two.lock() {
                    Ok(guard) => guard.reciev_from_client.clone(),
                    Err(_) => {
                        return;
                    }
                },
            };
            loop {
                let self_clone_two = Arc::clone(&system_clone_two);

                let mut lock = match self_clone_two.lock() {
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
                    Ok(client_message::ClientMessage::Publish {
                        topic_name,
                        payload: PayloadTypes::IncidentLocation(payload),
                        ..
                    }) => {
                        if topic_name == "incident" {
                            incident_location = Some(payload.get_incident().get_location());
                            drop(reciever); // Release the lock here

                            continue;
                        } else if topic_name == "incident_resolved" {
                            solved_incident_location = Some(payload.get_incident().get_location());
                            drop(reciever); // Release the lock here

                            continue;
                        }
                    }
                    Ok(_) => {
                        continue;
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
        });
        thread::spawn(move || {
            let pool = ThreadPool::new(10);
            let (tx, rx) = channel();
            watch_directory(Path::new(PATH).to_path_buf(), tx);
            let mut last_event_times: HashMap<String, Instant> = HashMap::new();

            loop {
                match rx.recv() {
                    Ok(event) => {
                        let now = Instant::now();
                        let should_process = match last_event_times.get(&event[0]) {
                            Some(&last_time) => {
                                now.duration_since(last_time)
                                    > Duration::from_secs(TIME_INTERVAL_IN_SECS)
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
        });
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
            "./camera_system/packets_config/publish_config_update.json",
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

        let camera = match system_clone.lock().unwrap().get_camera_by_id(camera_id) {
            Some(camera) => camera,
            None => {
                return Err(ProtocolError::InvalidCommand(
                    "Camera not found".to_string(),
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
        "./camera_system/packets_config/publish_incident_config.json",
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

