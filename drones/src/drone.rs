use chrono::{DateTime, Utc};
use client::client::Client;
use protocol::{client_message::{ClientMessage, Connect}, disconnect::disconnect_config::DisconnectConfig, messages_config::MessagesConfig, publish::{payload_types::PayloadTypes, publish_config::PublishConfig}, subscribe::{subscribe_config::SubscribeConfig, subscribe_properties::SubscribeProperties}};
use utils::{incident::Incident, incident_payload::IncidentPayload, location::Location, protocol_error::ProtocolError};

use std::{
    path::PathBuf, sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    }, thread::{self, sleep}, time::Duration
};

use super::{drone_config::DroneConfig, drone_error::DroneError, drone_state::DroneState};

use std::f64::consts::PI;
const LOW_BATERRY_LEVEL: i64 = 20;
pub const DRONE_SPEED: f64 = 0.001;
const TOLERANCE_FACTOR: f64 = 0.6;
const MILISECONDS_PER_SECOND: f64 = 1000.0;
const TWO_PI: f64 = 2.0 * PI;
const COORDINATE_SCALE_FACTOR: f64 = 100.0;
const FULL_BATTERY: i64 = 100;
const ANGLE_SCALING_FACTOR: f64 = 0.6; // este valor hace que en cada tick los drones avancen mas o menos

#[derive(Debug, Clone)]
#[allow(clippy::type_complexity)]
pub struct Drone {
    // ID unico para cada Drone.
    pub id: u32,

    ///posicion actual del Drone.
    pub location: Location,

    ///posicion del centro de drones al que pertenece.
    pub center_location: Location,

    /// La configuracion del Drone contiene el nivel de bateria del mismo y
    /// el radio de operacion.
    target_location: Location,

    updated_target_location: bool,

    /// Contiene configuraciones como la tasa de movimiento, la tasa de carga y descarga de bateria,
    /// y el radio de operacion.
    drone_config: DroneConfig,

    ///  El Drone puede tener distintos estados:
    /// - Waiting: esta circulando en su radio de operacion, pero no esta atendiendo ningun incidente.
    /// - AttendingIncident: un nuevo incidente fue cargado por la app de monitoreo, y el Drone fue asignado
    ///                         a resolverlo.
    /// - LowBatteryLevel: el Drone se quedo sin bateria, por lo que va a su central a cargarse, y no va a volver a
    ///                    funcionar hasta que tenga el nivel de bateria completo(al terminar de cargarse, vuelve a
    ///                    tener el estado Waiting).
    /// - ChargingBattery: se va a utilizar cuando el Drone este cargando su bateria en su central.
    ///                    La idea es que no patrulle ni se ponga a resolver incidentes en este estado.
    pub drone_state: DroneState,

    /// Client con el que va a interactuar en la red con las demas aplicaciones.
    pub drone_client: Client,

    /// Nivel de bateria actual del Drone.
    pub battery_level: i64,

    /// A traves de este sender, se envia la configuracion de los packets que el Drone
    /// quiera enviar a la red.
    send_to_client_channel:
        Arc<Mutex<Option<Sender<Box<dyn MessagesConfig + Send>>>>>,

    /// A traves de este receiver, el Drone recibe los mensajes provenientes de su Client.
    recieve_from_client: Arc<Mutex<Receiver<ClientMessage>>>,

    incidents: Vec<(Incident, u8)>,

    disconnect_receiver_from_center: Arc<Mutex<Receiver<()>>>,
}

impl Drone {
    /// levanto su configuracion, y me guardo su posicion inicial.
    pub fn new(
        id: u32,
        location: Location,
        center_location: Location,
        config_file_path: &str,
        address: String,
        disconnect_receiver_from_center: Receiver<()>,
    ) -> Result<Drone, DroneError> {
        
        let drone_config = DroneConfig::new(&config_file_path)?;

        let file = Drone::get_clean_path("packets_config/connect_config.json");

        let connect_config = Drone::read_connect_config(&file, id)?;

        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();
            println!("Drone {} created successfully", id);
        let drone_client = Drone::create_client(id, rx, address, connect_config, tx2, tx.clone())?;

       
        let target_location = location;
        Ok(Drone {
            id,
            location,
            center_location,
            target_location,
            updated_target_location: false,
            drone_config,
            drone_state: DroneState::Waiting,
            drone_client,
            battery_level: 100,
            send_to_client_channel: Arc::new(Mutex::new(Some(tx))),
            recieve_from_client: Arc::new(Mutex::new(rx2)),
            incidents: Vec::new(),
            disconnect_receiver_from_center: Arc::new(Mutex::new(disconnect_receiver_from_center)),
        })
    }

    /// Levanta la configuracion que va a tener el packet Connect que enviara el Client
    /// del Drone.
    ///
    /// Setea su client id al id que ingrese por parametro(va a estar definido por el usuario
    /// a la hora de instanciar a un nuevo Drone).
    pub fn read_connect_config(file_path: &str, id: u32) -> Result<Connect, DroneError> {
        let mut connect_config = match Connect::read_connect_config(file_path) {
            Ok(config) => config,
            Err(e) => {
                println!("path: {}", file_path);
                return Err(DroneError::ProtocolError(e.to_string()))},
        };
        connect_config.client_id = id.to_string();

        Ok(connect_config)
    }

    fn get_clean_path(path: &str) -> String {
        let project_dir = env!("CARGO_MANIFEST_DIR");
        let file_path = PathBuf::from(project_dir).join(path);
        println!("Test clients path: {:?}", file_path);  // Añade este print
        return file_path.to_str().unwrap().to_string();
    }

    /// Crea el Client a traves del cual el Drone va a comunicarse con la red.
    ///
    /// Este Client se va a construir a partir de la configuracion brindada en su archivo de configuracion.
    ///
    /// Una vez conectado, se envian packets que van a suscribir al Drone a sus topics de interes(ver metodo subscribe_to_topics).
    ///
    /// Retorna error en caso de fallar la creacion.
    fn create_client(
        id: u32,
        receive_from_drone_channel: Receiver<Box<dyn MessagesConfig + Send>>,
        address: String,
        connect_config: Connect,
        send_to_drone_channel: Sender<ClientMessage>,
        send_from_drone_channel: Sender<Box<dyn MessagesConfig + Send>>,
    ) -> Result<Client, DroneError> {
        match Client::new(
            receive_from_drone_channel,
            address,
            connect_config.clone(),
            send_to_drone_channel,
        ) {
            Ok(client) => {
                println!(
                    "I'm Drone {}, and my Client has been connected successfully!",
                    id
                );
                Drone::subscribe_to_topics(connect_config, send_from_drone_channel)?;
                Ok(client)
            }
            Err(e) => Err(DroneError::ProtocolError(e.to_string())),
        }
    }

    /// Contiene las subscripciones a los topics de interes para el Drone: necesita la suscripcion al topic
    /// de incidentes("incident"), y al de drones atendiendo incidentes("attending_incident"): la idea es que reciban los incidentes
    /// cargados desde la aplicacion de monitoreo, y que ademas sepan que drones estan atendiendo ciertos incidentes, para que los drones
    /// puedan organizarse para resolver los distintos incidentes.
    fn subscribe_to_topics(
        connect_config: Connect,
        send_from_drone_channel: Sender<Box<dyn MessagesConfig + Send>>,
    ) -> Result<(), DroneError> {
        let subscribe_properties =
            SubscribeProperties::new(1, connect_config.properties.user_properties);
        let topics = vec!["incident", "attending_incident"];

        for topic_name in topics {
            Self::subscribe_to_topic(
                &topic_name.to_string(),
                &subscribe_properties,
                &connect_config.client_id,
                &send_from_drone_channel,
            )?;
        }

        Ok(())
    }

    fn subscribe_to_topic(
        topic_name: &String,
        subscribe_properties: &SubscribeProperties,
        client_id: &String,
        send_from_channel: &Sender<Box<dyn MessagesConfig + Send>>,
    ) -> Result<(), DroneError> {
        let subscribe_config = SubscribeConfig::new(
            topic_name.clone(),
            subscribe_properties.clone(),
            client_id.clone(),
        );

        match send_from_channel.send(Box::new(subscribe_config)) {
            Ok(_) => {
                println!(
                    "Drone {} suscribed to topic {} successfully",
                    client_id, topic_name
                );
                Ok(())
            }
            Err(e) => {
                println!("Drone {}: Error sending message: {:?}", client_id, e);
                Err(DroneError::SubscribeError(e.to_string()))
            }
        }
    }

    /// Desconecta al Client del Drone. Se cierra el canal por el cual el Drone envia packets
    /// a su Client, y ademas se desconecta a su Client.
    pub fn disconnect(&mut self) -> Result<(), ProtocolError> {
        let disconnect_config =
            DisconnectConfig::new(0x00_u8, 1, "normal".to_string(), self.id.to_string());
        let send_to_client_channel = match self.send_to_client_channel.lock() {
            Ok(channel) => channel,
            Err(_) => {
                return Err(ProtocolError::LockError);
            }
        };

        if let Some(ref sender) = *send_to_client_channel {
            match sender.send(Box::new(disconnect_config)) {
                Ok(_) => {
                    println!("Drone {} disconnected successfully", self.id);
                }
                Err(e) => {
                    println!("Error sending to client channel: {:?}", e);
                    return Err(ProtocolError::SendError(e.to_string()));
                }
            };
        }

        println!("Drone client {} disconnected successfully", self.id);
        Ok(())
    }

    /// Esta funcion ejecuta el Drone, creando tres threads:
    /// - Uno para manejar la bateria del Drone: Mientras tenga un estado de Waiting, se considera
    ///   que tiene un nivel de bateria optimo para operar: cuando este en este estado, su bateria se ira
    ///   descargando a medida que pasa el tiempo(la tasa de descarga de bateria viene en el archivo de
    ///   configuracion del Drone). Cuando se llega a un estado de LowBatteryLevel, se notifica al thread de movimiento
    ///   la necesidad de ir a cargarse a la central de carga. Una vez en ella, el Drone pasa a cargarse con un estado de
    ///   ChargingBattery(la tasa de carga de bateria viene en el archivo de configuracion del Drone).
    ///
    /// - Un segundo thread para mover al Drone: si esta en estado Waiting, va a patrullar
    ///   sobre su radio de operacion; si esta en estado de LowBatteryLevel, se mueve
    ///   hacia su central de carga; y si esta en estado de AttendingIncident, se mueve hacia la
    ///   localizacion del incidente a resolver.
    ///
    /// - Un tercer thread para manejar la recepcion de mensajes de parte de su Client:
    ///   recibe los Publish packets que vengan de los topics al que este suscrito.
    pub fn run_drone(&mut self) -> Result<(), DroneError> {
        match self.drone_client.client_run() {
            Ok(_) => println!("Drone {} patrolling in its area of operation", self.id),
            Err(e) => {
                print!(
                    "Error while running the Drone Client with id {}: {:?}",
                    self.id, e
                );
                return Err(DroneError::ProtocolError(e.to_string()));
            }
        };

        let (disconnect_sender, disconnect_receiver) = mpsc::channel();
        let (disconnect_sender_two, disconnect_receiver_two) = mpsc::channel();
        let (disconnect_sender_three, disconnect_receiver_three) = mpsc::channel();

        let disconnect_sender_clone = disconnect_sender.clone();
        let disconnect_sender_two_clone = disconnect_sender.clone();

        let drone_ref = Arc::new(Mutex::new(self.clone()));
        let self_clone_one = Arc::clone(&drone_ref);
        let self_clone_two = Arc::clone(&drone_ref);
        let self_clone_three = Arc::clone(&drone_ref);
        let recieve_from_client_clone = Arc::clone(&self.recieve_from_client);
        let send_to_client_channel_clone = Arc::clone(&self.send_to_client_channel);
        let disconnect_receiver_from_center_clone =
            Arc::clone(&self.disconnect_receiver_from_center);

        thread::spawn(move || loop {
            let lock = disconnect_receiver_from_center_clone.lock().unwrap();
            if lock.try_recv().is_ok() {
                match disconnect_sender_clone.send(()) {
                    Ok(_) => (),
                    Err(e) => eprint!("{}", e),
                };

                match disconnect_sender_two_clone.send(()) {
                    Ok(_) => (),
                    Err(e) => eprint!("{}", e),
                };

                match disconnect_sender_three.send(()) {
                    Ok(_) => (),
                    Err(e) => eprint!("{}", e),
                };
            }
        });

        thread::spawn(move || {
            match Drone::handle_battery_changes(self_clone_one, disconnect_receiver) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error handling battery changes: {:?}", e);
                }
            }
        });

        thread::spawn(move || {
            match Drone::handle_drone_movement(self_clone_two, disconnect_receiver_two) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error handling drone movement: {:?}", e);
                }
            }
        });

        thread::spawn(move || {
            match Drone::handle_message_reception_from_client(
                self_clone_three,
                recieve_from_client_clone,
                send_to_client_channel_clone,
                disconnect_sender,
                disconnect_sender_two,
                disconnect_receiver_three,
            ) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error handling message reception from client: {:?}", e);
                }
            }
        });

        Ok(())
    }

    /// Closure del thread que controla la bateria del Drone.
    ///
    /// La idea es que cuando este en estado Waiting, se descargue a medida que pase
    /// el tiempo la bateria del Drone, y cuando se pase a estado de ChargingBattery,
    /// la bateria comience a cargarse.
    fn handle_battery_changes(
        drone_ref: Arc<Mutex<Drone>>,
        disconnect_receiver: Receiver<()>,
    ) -> Result<(), DroneError> {
        let mut last_discharge_time = Utc::now();

        loop {
            if disconnect_receiver.try_recv().is_ok() {
                return Ok(());
            }

            let self_clone = Arc::clone(&drone_ref);
            let mut lock = match self_clone.lock() {
                Ok(locked) => locked,
                Err(e) => {
                    println!("Error locking drone: {:?}", e);
                    return Err(DroneError::LockError(e.to_string()));
                }
            };

            match lock.drone_state.clone() {
                DroneState::Waiting => {
                    let updated_last_discharge_time = lock.battery_discharge(last_discharge_time);
                    last_discharge_time = updated_last_discharge_time;
                }

                DroneState::ChargingBattery => match lock.charge_battery() {
                    Ok(_) => {
                        println!("Drone {} charged its battery: returning to patrol", lock.id);
                    }
                    Err(e) => {
                        println!("Error charging battery: {:?}", e);
                        return Err(DroneError::ChargingBatteryError(e.to_string()));
                    }
                },
                _ => (),
            }
        }
    }

    /// Closure del thread de movimiento del Drone.
    ///
    /// Al estar con estado de Waiting, el Drone patrullara dentro de su radio de operacion hasta ser notificado de un nuevo
    /// incidente, o si se queda con niveles bajos de bateria.
    ///
    /// Al registrar un incidente, el Drone se dirige a resolverlo.
    ///
    /// Al quedar con niveles bajos de bateria, el Drone se movera hacia su central de operacion para cargarse.
    fn handle_drone_movement(
        drone_ref: Arc<Mutex<Drone>>,
        disconnect_receiver: Receiver<()>,
    ) -> Result<(), DroneError> {
        loop {
            if disconnect_receiver.try_recv().is_ok() {
                return Ok(());
            }

            sleep(Duration::from_millis(500));

            let self_clone = Arc::clone(&drone_ref);
            let mut lock = match self_clone.lock() {
                Ok(locked) => locked,
                Err(e) => {
                    println!("Error locking drone: {:?}", e);
                    return Err(DroneError::LockError(e.to_string()));
                }
            };
            match lock.drone_state.clone() {
                DroneState::Waiting => {
                    match lock.patrolling_in_operating_radius() {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Error while patrolling: {:?}", e);
                            return Err(DroneError::PatrollingError(e.to_string()));
                        }
                    };
                }
                DroneState::AttendingIncident(location) => {
                    println!("Drone {} yendo a solucionar el incidente", lock.id);
                    lock.update_target_location(Some(location))?;
                    if lock.calculate_new_position() {
                        lock.publish_attending_accident(location);
                    }
                }
                DroneState::LowBatteryLevel => {
                    lock.redirect_to_operation_center()?;
                }
                _ => (),
            };
        }
    }

    /// closure del thread de recepcion de mensajes de parte del Client del Drone.
    ///
    /// Por cada mensaje recibido de los topics de interes del Drone("incident" y "attending_incident"),
    /// el Drone determinara su accionar.
    ///
    /// Si recibe un incidente, se redirige hacia el mismo para resolverlo y publica su nuevo estado en
    /// attending_incident.
    /// A su vez, recibe aquellas notificaciones de todos los Drones que esten yendo a resolver el incidente, y van a jugar una carrera:
    /// los primeros 2 Drones que lleguen, se podran a resolver el incidente, y los demas pasaran a ignorar este incidente y volveran a patrullar.

    #[allow(clippy::type_complexity)]
    fn handle_message_reception_from_client(
        drone_ref: Arc<Mutex<Drone>>,
        receive_from_client_ref: Arc<Mutex<Receiver<ClientMessage>>>,
        send_to_client_channel: Arc<
            Mutex<Option<Sender<Box<dyn MessagesConfig + Send>>>>,
        >,
        disconnect_sender: Sender<()>,
        disconnect_sender_two: Sender<()>,
        disconnect_receiver_from_center: Receiver<()>,
    ) -> Result<(), DroneError> {
        loop {
            if let Ok(()) = disconnect_receiver_from_center.try_recv() {
                return Ok(());
            }

            let message = match receive_from_client_ref.lock() {
                Ok(lock) => match lock.recv() {
                    Ok(msg) => msg,
                    Err(e) => {
                        return Err(DroneError::ReceiveError(e.to_string()));
                    }
                },
                Err(e) => {
                    return Err(DroneError::LockError(e.to_string()));
                }
            };

            let mut self_cloned = drone_ref.lock().expect("Error locking drone");

            match message {
                ClientMessage::Publish {
                    topic_name,
                    payload: PayloadTypes::IncidentLocation(payload),
                    ..
                } => match topic_name.as_str() {
                    "incident" => {
                        if self_cloned.drone_state == DroneState::Waiting {
                            let location = payload.get_incident().get_location();
                            self_cloned.drone_state = DroneState::AttendingIncident(location);

                            let (incident, drones_attending_incident) =
                                (payload.get_incident().clone(), 0);

                            self_cloned
                                .incidents
                                .push((incident.clone(), drones_attending_incident));
                        }
                    }

                    _ => continue,
                },
                ClientMessage::Publish {
                    topic_name,
                    payload: PayloadTypes::AttendingIncident(payload),
                    ..
                } => match topic_name.as_str() {
                    "attending_incident" => {
                        let mut to_remove = Vec::new();

                        for (incident, count) in self_cloned.incidents.iter_mut() {
                            if incident.get_location() == payload.get_incident().get_location() {
                                *count += 1;

                                if *count == 2 {
                                    sleep(Duration::from_secs(10));
                                    to_remove.push(incident.clone());

                                    let incident_payload = IncidentPayload::new(Incident::new(
                                        incident.get_location(),
                                    ));
                                    let publish_config = match PublishConfig::read_config(
                                        "drones/packets_config/publish_solved_incident_config.json",
                                        PayloadTypes::IncidentLocation(incident_payload),
                                    ) {
                                        Ok(config) => config,
                                        Err(e) => {
                                            println!("Error reading publish config: {:?}", e);
                                            return Err(DroneError::ProtocolError(e.to_string()));
                                        }
                                    };
                                    let send_to_client_channel = match send_to_client_channel.lock()
                                    {
                                        Ok(channel) => channel,
                                        Err(e) => {
                                            println!(
                                                "Error locking send_to_client_channel: {:?}",
                                                e
                                            );
                                            return Err(DroneError::LockError(e.to_string()));
                                        }
                                    };

                                    if let Some(ref sender) = *send_to_client_channel {
                                        match sender.send(Box::new(publish_config)) {
                                            Ok(_) => {
                                                println!("Drone notifies the resolution of the incident at the location {:?}", incident.get_location());
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
                        }

                        if !to_remove.is_empty() {
                            for incident in to_remove {
                                self_cloned.incidents.retain(|(i, _)| i != &incident);
                            }
                            self_cloned.drone_state = DroneState::Waiting;
                        }
                    }
                    _ => continue,
                },
                ClientMessage::Disconnect {
                    reason_code: _,
                    session_expiry_interval: _,
                    reason_string: _,
                    client_id: _,
                } => {
                    match disconnect_sender.send(()) {
                        Ok(_) => (),
                        Err(e) => return Err(DroneError::SendError(e.to_string())),
                    };
                    match disconnect_sender_two.send(()) {
                        Ok(_) => (),
                        Err(e) => return Err(DroneError::SendError(e.to_string())),
                    };
                }
                _ => {}
            };
        }
    }

    pub fn get_state(self) -> DroneState {
        self.drone_state
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    /// Carga un 1% la bateria si el elapsed time (diferencial de tiempo inicial y tiempo final)
    /// es mayor o igual al ratio de carga
    ///
    /// Si la bateria llega a 100%, el estado del dron pasa a Waiting
    pub fn update_battery_charge(
        &mut self,
        start_time: DateTime<Utc>,
        current_time: DateTime<Utc>,
    ) -> (DateTime<Utc>, bool) {
        let elapsed_time = current_time
            .signed_duration_since(start_time)
            .num_milliseconds();
        let charge_rate = self.drone_config.get_battery_charge_rate_milisecs();

        if elapsed_time >= charge_rate {
            self.battery_level += 1;
            if self.battery_level >= FULL_BATTERY - 1 {
                self.battery_level = FULL_BATTERY;
                self.drone_state = DroneState::Waiting;
            }
            (current_time, true)
        } else {
            (start_time, false)
        }
    }

    /// Descarga un 1% la bateria si el elapsed time (diferencial de tiempo inicial y tiempo final)
    /// es mayor o igual al ratio de descarga
    /// Si la bateria llega a 20%, el estado del dron pasa a LowBatteryLevel
    pub fn update_battery_discharge(
        &mut self,
        last_discharge_time: DateTime<Utc>,
        current_time: DateTime<Utc>,
    ) -> (DateTime<Utc>, bool) {
        let elapsed_time = current_time
            .signed_duration_since(last_discharge_time)
            .num_milliseconds();
        let discharge_rate = self.drone_config.get_battery_discharge_rate();

        if elapsed_time >= discharge_rate {
            self.battery_level -= 1;
            if self.battery_level == LOW_BATERRY_LEVEL {
                println!("Drone {} was left with low battery levels", self.id);
                self.drone_state = DroneState::LowBatteryLevel;
            }
            (current_time, true)
        } else {
            (last_discharge_time, false)
        }
    }

    /// Se descarga el nivel de bateria del
    /// Drone segun indique la tasa de descarga de bateria del mismo(definida
    /// en la config del Drone).
    ///
    /// Cada vez que se cumpla "un ciclo" de la tasa de descarga, se reduce la bateria del
    /// Drone en un 1%.
    pub fn battery_discharge(&mut self, last_discharge_time: DateTime<Utc>) -> DateTime<Utc> {
        let current_time = Utc::now();
        let (updated_last_discharge_time, _updated) =
            self.update_battery_discharge(last_discharge_time, current_time);

        updated_last_discharge_time
    }

    /// Se carga el nivel de bateria del
    /// Drone segun indique la tasa de carga de bateria del mismo(definida
    /// en la config del Drone).
    ///
    /// Cada vez que se cumpla "un ciclo" de la tasa de carga, se aumenta la bateria del
    /// Drone en un 1%.
    pub fn battery_charge(&mut self, last_charge_time: DateTime<Utc>) -> DateTime<Utc> {
        let current_time = Utc::now();
        let (updated_last_charge_time, _updated) =
            self.update_battery_charge(last_charge_time, current_time);

        updated_last_charge_time
    }

    /// Carga al Drone de acuerdo a la tasa de carga que venga definida en la configuracion.
    ///
    /// Al llegar al 100%, devuelve un DroneState del tipo Waiting.
    pub fn charge_battery(&mut self) -> Result<DroneState, DroneError> {
        println!("Drone {} charging its battery at the central", self.id);
        let mut start_time = Utc::now();

        loop {
            let current_time = Utc::now();
            let (updated_start_time, updated) =
                self.update_battery_charge(start_time, current_time);

            if updated && self.battery_level > FULL_BATTERY - 1 {
                return Ok(self.drone_state.clone());
            }
            start_time = updated_start_time;
        }
    }

    /// Cuando el Drone esta en estado Waiting, lo que hace es patrullar alrededor
    /// de su radio de operacion.
    ///
    /// Esto se logra usando update_target_location, que calcula una nueva posicion
    /// para seguir "dentro de su circulo".
    ///
    /// La idea es que si se llega al current_target_location, el Drone calcule una nueva
    /// posicion para ir, y que comience a moverse.
    fn patrolling_in_operating_radius(&mut self) -> Result<(), DroneError> {
        if (self.location.lat * COORDINATE_SCALE_FACTOR) / COORDINATE_SCALE_FACTOR
            == (self.target_location.lat * COORDINATE_SCALE_FACTOR) / COORDINATE_SCALE_FACTOR
            && (self.location.long * COORDINATE_SCALE_FACTOR) / COORDINATE_SCALE_FACTOR
                == (self.target_location.long * COORDINATE_SCALE_FACTOR) / COORDINATE_SCALE_FACTOR
        {
            self.update_target_location(None)?;
        }

        self.calculate_new_position();

        Ok(())
    }

    /// Una vez que el Drone entra en estado de LowBatteryLevel,
    /// se redirecciona hacia su centro de carga.
    ///
    /// Una vez que llega a esta location, cambia su estado a
    /// ChargingBattery, por lo que el Drone va a comenzar a cargarse.
    fn redirect_to_operation_center(&mut self) -> Result<(), DroneError> {
        println!("Drone {} redirigiendose a su central de carga", self.id);

        if self.location == self.center_location {
            self.drone_state = DroneState::ChargingBattery;
        }
        if self.target_location != self.center_location {
            self.update_target_location(Some(self.center_location))?;
        }
        self.calculate_new_position();
        Ok(())
    }

    pub fn calculate_new_position(&mut self) -> bool {
        let direction_lat = self.target_location.lat - self.location.lat;
        let direction_long = self.target_location.long - self.location.long;
        let magnitude = (direction_lat.powi(2) + direction_long.powi(2)).sqrt();
        let effective_range = DRONE_SPEED * TOLERANCE_FACTOR;
        if magnitude < effective_range {
            self.location = self.target_location;
            return true;
        }
        let unit_direction_lat = direction_lat / magnitude;
        let unit_direction_long = direction_long / magnitude;

        let new_lat = self.location.lat + unit_direction_lat * DRONE_SPEED;
        let new_long = self.location.long + unit_direction_long * DRONE_SPEED;

        self.location = Location::new(new_lat, new_long);
        false
    }

    fn update_location(&mut self) {
        let publish_config = match PublishConfig::read_config(
            "drones/packets_config/publish_config.json",
            PayloadTypes::DroneLocation(self.id, self.location, self.target_location),
        ) {
            Ok(config) => config,
            Err(e) => {
                println!("Error reading publish config: {:?}", e);
                return;
            }
        };

        let lock = match self.send_to_client_channel.lock() {
            Ok(lock) => lock,
            Err(e) => {
                println!("Error locking send_to_client_channel: {:?}", e);
                return;
            }
        };
        if let Some(ref sender) = *lock {
            match sender.send(Box::new(publish_config)) {
                Ok(_) => self.updated_target_location = true,
                Err(e) => {
                    println!("Error sending to client channel: {:?}", e);
                }
            };
        }
    }

    fn publish_attending_accident(&mut self, location: Location) {
        let incident = Incident::new(location);
        let incident_payload = IncidentPayload::new(incident.clone());
        let publish_config = match PublishConfig::read_config(
            "drones/packets_config/publish_attending_incident_config.json",
            PayloadTypes::AttendingIncident(incident_payload),
        ) {
            Ok(config) => config,
            Err(e) => {
                println!("Error reading publish config: {:?}", e);
                return;
            }
        };

        let lock = match self.send_to_client_channel.lock() {
            Ok(lock) => lock,
            Err(e) => {
                println!("Error locking send_to_client_channel: {:?}", e);
                return;
            }
        };
        if let Some(ref sender) = *lock {
            match sender.send(Box::new(publish_config)) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error sending to client channel: {:?}", e);
                }
            };
        }
    }
    fn update_target_location(
        &mut self,
        target_location: Option<Location>,
    ) -> Result<(), DroneError> {
        match target_location {
            Some(location) => {
                self.target_location = location;
            }
            None => {
                let current_time = Utc::now().timestamp_millis() as f64;
                let angle =
                    ((current_time * ANGLE_SCALING_FACTOR) / MILISECONDS_PER_SECOND) % TWO_PI;
                let operation_radius = self.drone_config.get_operation_radius();
                let new_target_lat = self.center_location.lat + operation_radius * angle.cos();
                let new_target_long = self.center_location.long + operation_radius * angle.sin();
                self.target_location = Location::new(new_target_lat, new_target_long);
            }
        }
        self.update_location();
        Ok(())
    }
}

