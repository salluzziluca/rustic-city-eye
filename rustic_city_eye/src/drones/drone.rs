use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, sleep},
    time::Duration,
};

use super::{drone_config::DroneConfig, drone_error::DroneError, drone_state::DroneState};
use crate::{
    monitoring::incident::Incident,
    mqtt::{
        client::Client,
        client_message::{self, ClientMessage, Connect},
        disconnect_config::DisconnectConfig,
        messages_config::{self, MessagesConfig},
        protocol_error::ProtocolError,
        publish::publish_config::PublishConfig,
        subscribe_config::SubscribeConfig,
        subscribe_properties::SubscribeProperties,
    },
    utils::{incident_payload::IncidentPayload, location::Location, payload_types::PayloadTypes},
};
use chrono::{DateTime, Utc};
use std::f64::consts::PI;
const LOW_BATERRY_LEVEL: i64 = 20;
const DRONE_SPEED: f64 = 0.001;
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
        Arc<Mutex<Option<Sender<Box<dyn messages_config::MessagesConfig + Send>>>>>,

    /// A traves de este receiver, el Drone recibe los mensajes provenientes de su Client.
    recieve_from_client: Arc<Mutex<Receiver<ClientMessage>>>,

    incidents: Vec<(Incident, u8)>,
}

impl Drone {
    /// levanto su configuracion, y me guardo su posicion inicial.
    pub fn new(
        id: u32,
        location: Location,
        center_location: Location,
        config_file_path: &str,
        address: String,
    ) -> Result<Drone, DroneError> {
        let drone_config = DroneConfig::new(config_file_path)?;
        let connect_config = Drone::read_connect_config("src/drones/connect_config.json", id)?;

        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        let drone_client = Drone::create_client(id, rx, address, connect_config, tx2, tx.clone())?;

        let target_location = location;
        Ok(Drone {
            id,
            location,
            center_location,
            target_location,
            drone_config,
            drone_state: DroneState::Waiting,
            drone_client,
            battery_level: 100,
            send_to_client_channel: Arc::new(Mutex::new(Some(tx))),
            recieve_from_client: Arc::new(Mutex::new(rx2)),
            incidents: Vec::new(),
        })
    }

    /// Levanta la configuracion que va a tener el packet Connect que enviara el Client
    /// del Drone.
    ///
    /// Setea su client id al id que ingrese por parametro(va a estar definido por el usuario
    /// a la hora de instanciar a un nuevo Drone).
    fn read_connect_config(file_path: &str, id: u32) -> Result<Connect, DroneError> {
        let mut connect_config = match Connect::read_connect_config(file_path) {
            Ok(config) => config,
            Err(e) => return Err(DroneError::ProtocolError(e.to_string())),
        };
        connect_config.client_id = id.to_string();

        Ok(connect_config)
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
                println!("Soy el Drone {}, y mi Client se conecto exitosamente!", id);
                Drone::subscribe_to_topics(connect_config, send_from_drone_channel)?;
                Ok(client)
            }
            Err(e) => Err(DroneError::ProtocolError(e.to_string())),
        }
    }

    /// Contiene las subscripciones a los topics de interes para el Drone: necesita la suscripcion al topic
    /// de incidentes("incident"), y al de drones atendiendo incidentes("attendingincident"): la idea es que reciban los incidentes
    /// cargados desde la aplicacion de monitoreo, y que ademas sepan que drones estan atendiendo ciertos incidentes, para que los drones
    /// puedan organizarse para resolver los distintos incidentes.
    fn subscribe_to_topics(
        connect_config: Connect,
        send_from_drone_channel: Sender<Box<dyn MessagesConfig + Send>>,
    ) -> Result<(), DroneError> {
        let subscribe_properties =
            SubscribeProperties::new(1, connect_config.properties.user_properties);
        let topic_name = "incidente".to_string();
        let subscribe_config = SubscribeConfig::new(
            topic_name.clone(),
            subscribe_properties.clone(),
            connect_config.client_id.clone(),
        );

        match send_from_drone_channel.send(Box::new(subscribe_config)) {
            Ok(_) => {
                println!(
                    "Drone {} suscrito al topic {} correctamente",
                    connect_config.client_id, topic_name
                );
            }
            Err(e) => {
                println!(
                    "Drone {}: Error sending message: {:?}",
                    connect_config.client_id, e
                );
                return Err(DroneError::SubscribeError(e.to_string()));
            }
        };

        let topic_name = "attendingincident".to_string();
        let subscribe_config = SubscribeConfig::new(
            topic_name.clone(),
            subscribe_properties,
            connect_config.client_id.clone(),
        );
        match send_from_drone_channel.send(Box::new(subscribe_config)) {
            Ok(_) => {
                println!(
                    "Drone {} suscrito al topic {} correctamente",
                    connect_config.client_id, topic_name
                );
            }
            Err(e) => {
                println!(
                    "Drone {}: Error sending message: {:?}",
                    connect_config.client_id, e
                );
                return Err(DroneError::SubscribeError(e.to_string()));
            }
        };

        Ok(())
    }

    /// Desconecta al Client del Drone. Se cierra el canal por el cual el Drone envia packets
    /// a su Client, y ademas se desconecta a su Client.
    pub fn disconnect(&mut self) -> Result<(), ProtocolError> {
        let disconnect_config =
            DisconnectConfig::new(0x00_u8, 1, "normal".to_string(), self.id.to_string());
        let send_to_client_channel = match self.send_to_client_channel.lock() {
            Ok(channel) => channel,
            Err(e) => {
                println!("Error locking send_to_client_channel: {:?}", e);
                return Err(ProtocolError::LockError);
            }
        };
        if let Some(ref sender) = *send_to_client_channel {
            match sender.send(Box::new(disconnect_config)) {
                Ok(_) => {
                    println!("Drone {} desconectado correctamente", self.id);
                }
                Err(e) => {
                    println!("Error sending to client channel: {:?}", e);
                    return Err(ProtocolError::SendError(e.to_string()));
                }
            };
        }

        println!("Cliente del drone {} desconectado correctamente", self.id);
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
            Ok(client) => {
                println!("Drone {} patrullando en su area de operacion", self.id);
                client
            }
            Err(e) => {
                print!(
                    "Error al correr el Client del Drone con id {}: {:?}",
                    self.id, e
                );
                return Err(DroneError::ProtocolError(e.to_string()));
            }
        };

        let drone_ref = Arc::new(Mutex::new(self.clone()));
        let self_clone_one = Arc::clone(&drone_ref);
        let self_clone_two = Arc::clone(&drone_ref);
        let self_clone_three = Arc::clone(&drone_ref);
        let recieve_from_client_clone = Arc::clone(&self.recieve_from_client);
        let send_to_client_channel_clone = Arc::clone(&self.send_to_client_channel);

        thread::spawn(
            move || match Drone::handle_battery_changes(self_clone_one) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error handling battery changes: {:?}", e);
                }
            },
        );

        thread::spawn(move || match Drone::handle_drone_movement(self_clone_two) {
            Ok(_) => (),
            Err(e) => {
                println!("Error handling drone movement: {:?}", e);
            }
        });

        thread::spawn(move || {
            match Drone::handle_message_reception_from_client(
                self_clone_three,
                recieve_from_client_clone,
                send_to_client_channel_clone,
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
    fn handle_battery_changes(drone_ref: Arc<Mutex<Drone>>) -> Result<(), DroneError> {
        let mut last_discharge_time = Utc::now();

        loop {
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
                        println!(
                            "El Drone {} cargo su bateria: volviendo a patrullar",
                            lock.id
                        );
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
    fn handle_drone_movement(drone_ref: Arc<Mutex<Drone>>) -> Result<(), DroneError> {
        loop {
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
                    match lock.drone_movement(location) {
                        Ok(is_at_incident_location) => {
                            if is_at_incident_location {
                                lock.publish_attending_accident(location);
                            }
                        }
                        Err(e) => {
                            println!("Error while moving to incident: {:?}", e);
                            return Err(DroneError::MovingToIncidentError(e.to_string()));
                        }
                    };
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
    /// Por cada mensaje recibido de los topics de interes del Drone("incident" y "attendingincident"),
    /// el Drone determinara su accionar.
    ///
    /// Si recibe un incidente, se redirige hacia el mismo para resolverlo y publica su nuevo estado en
    /// attendingincident.
    /// A su vez, recibe aquellas notificaciones de todos los Drones que esten yendo a resolver el incidente, y van a jugar una carrera:
    /// los primeros 2 Drones que lleguen, se podran a resolver el incidente, y los demas pasaran a ignorar este incidente y volveran a patrullar.
    fn handle_message_reception_from_client(
        drone_ref: Arc<Mutex<Drone>>,
        receive_from_client_ref: Arc<Mutex<Receiver<ClientMessage>>>,
        send_to_client_channel: Arc<
            Mutex<Option<Sender<Box<dyn messages_config::MessagesConfig + Send>>>>,
        >,
    ) -> Result<(), DroneError> {
        loop {
            let message = match receive_from_client_ref.lock() {
                Ok(lock) => match lock.recv() {
                    Ok(msg) => msg, // Successfully received a message
                    Err(e) => {
                        return Err(DroneError::ReceiveError(e.to_string()));
                    }
                },
                Err(e) => {
                    return Err(DroneError::LockError(e.to_string()));
                }
            };

            let mut self_cloned = match drone_ref.lock() {
                Ok(locked) => locked,
                Err(e) => {
                    return Err(DroneError::LockError(e.to_string()));
                }
            };

            if let client_message::ClientMessage::Publish {
                topic_name,
                payload: PayloadTypes::IncidentLocation(payload),
                ..
            } = message
            {
                match topic_name.as_str() {
                    "incidente" => {
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
                }
            } else if let client_message::ClientMessage::Publish {
                topic_name,
                payload: PayloadTypes::AttendingIncident(payload),
                ..
            } = message
            {
                match topic_name.as_str() {
                    "attendingincident" => {
                        Drone::handle_attending_incident(
                            &drone_ref,
                            payload,
                            &send_to_client_channel,
                        )?;
                    }
                    _ => continue,
                }
            }
        }
    }

    fn handle_attending_incident(
        drone_ref: &Arc<Mutex<Drone>>,
        payload: IncidentPayload,
        send_to_client_channel: &Arc<
            Mutex<Option<Sender<Box<dyn messages_config::MessagesConfig + Send>>>>,
        >,
    ) -> Result<(), DroneError> {
        let mut self_cloned = match drone_ref.lock() {
            Ok(locked) => locked,
            Err(e) => {
                return Err(DroneError::LockError(e.to_string()));
            }
        };

        let mut to_remove = Vec::new();

        for (incident, count) in self_cloned.incidents.iter_mut() {
            if incident.get_location() == payload.get_incident().get_location() {
                *count += 1;

                if *count == 2 {
                    sleep(Duration::from_secs(10));
                    to_remove.push(incident.clone());

                    let incident_payload =
                        IncidentPayload::new(Incident::new(incident.get_location()));
                    let publish_config = match PublishConfig::read_config(
                        "src/monitoring/publish_solved_incident_config.json",
                        PayloadTypes::IncidentLocation(incident_payload),
                    ) {
                        Ok(config) => config,
                        Err(e) => {
                            println!("Error reading publish config: {:?}", e);
                            return Err(DroneError::ProtocolError(e.to_string()));
                        }
                    };
                    let send_to_client_channel = match send_to_client_channel.lock() {
                        Ok(channel) => channel,
                        Err(e) => {
                            println!("Error locking send_to_client_channel: {:?}", e);
                            return Err(DroneError::LockError(e.to_string()));
                        }
                    };

                    if let Some(ref sender) = *send_to_client_channel {
                        match sender.send(Box::new(publish_config)) {
                            Ok(_) => {
                                println!("Drone notifica la resolucion del incidente en la location {:?}", incident.get_location());
                            }
                            Err(e) => {
                                println!("Error sending to client channel: {:?}", e);
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

        Ok(())
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
    fn update_battery_charge(
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
    fn update_battery_discharge(
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
                println!("El Drone {} se quedo con niveles de bateria bajos", self.id);
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
        println!(
            "El Drone {} esta cargando su bateria en la central",
            self.id
        );
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
            self.update_target_location()?;
        }

        let (new_lat, new_long) = self.calculate_new_position(
            0.001,
            &self.location.lat,
            &self.location.long,
            &self.target_location.lat,
            &self.target_location.long,
        );
        self.location.lat = new_lat;
        self.location.long = new_long;

        self.update_location();

        Ok(())
    }

    /// Una vez que el Drone entra en estado de LowBatteryLevel,
    /// se redirecciona hacia su centro de carga.
    ///
    /// Una vez que llega a esta location, cambia su estado a
    /// ChargingBattery, por lo que el Drone va a comenzar a cargarse.
    fn redirect_to_operation_center(&mut self) -> Result<(), DroneError> {
        println!("Drone {} redirigiendose a su central de carga", self.id);
        if (self.location.lat * COORDINATE_SCALE_FACTOR) / COORDINATE_SCALE_FACTOR
            == (self.center_location.lat * COORDINATE_SCALE_FACTOR) / COORDINATE_SCALE_FACTOR
            && (self.location.long * COORDINATE_SCALE_FACTOR) / COORDINATE_SCALE_FACTOR
                == (self.center_location.long * COORDINATE_SCALE_FACTOR) / COORDINATE_SCALE_FACTOR
        {
            self.drone_state = DroneState::ChargingBattery;
        }

        self.update_drone_position(self.center_location)?;

        Ok(())
    }
    /// Mueve al Drone hacia la location que se le pase por parametro.
    /// Si la ubicacion actual del drone coincide con la de la target location
    /// (teniendo en cuenta un redondeo estipulado por el COORDINATE_SCALE_FACTOR)
    /// devuelve true
    fn drone_movement(&mut self, target_location: Location) -> Result<bool, DroneError> {
        if (self.location.lat * COORDINATE_SCALE_FACTOR) / COORDINATE_SCALE_FACTOR
            == (target_location.lat * COORDINATE_SCALE_FACTOR) / COORDINATE_SCALE_FACTOR
            && (self.location.long * COORDINATE_SCALE_FACTOR) / COORDINATE_SCALE_FACTOR
                == (target_location.long * COORDINATE_SCALE_FACTOR) / COORDINATE_SCALE_FACTOR
        {
            return Ok(true);
        }

        self.update_drone_position(target_location)?;

        Ok(false)
    }

    /// Actualiza la posicion del Drone segun la location que se le pase por parametro.
    /// Calcula la nueva posicion del Drone segun la velocidad de movimiento del mismo.
    /// Envia la nueva posicion al broker mediante un publish message
    fn update_drone_position(&mut self, target_location: Location) -> Result<(), DroneError> {
        let (new_lat, new_long) = self.calculate_new_position(
            DRONE_SPEED,
            &self.location.lat,
            &self.location.long,
            &target_location.lat,
            &target_location.long,
        );
        self.location.lat = new_lat;
        self.location.long = new_long;

        self.update_location();
        Ok(())
    }

    /// Calcula la nueva posicion del Drone segun la velocidad de movimiento del mismo.
    /// Si la nueva posicion está a menos de cierto rango de la `target_location`
    /// (definida por el tolerance factor multiplicado por la velocidad)
    /// se considera que el dron ha llegado a su destino
    fn calculate_new_position(
        &self,
        speed: f64,
        current_lat: &f64,
        current_long: &f64,
        target_lat: &f64,
        target_long: &f64,
    ) -> (f64, f64) {
        let direction_lat = target_lat - current_lat;
        let direction_long = target_long - current_long;
        let magnitude = (direction_lat.powi(2) + direction_long.powi(2)).sqrt();
        let effective_range = speed * TOLERANCE_FACTOR;
        if magnitude < effective_range {
            return (*target_lat, *target_long);
        }
        let unit_direction_lat = direction_lat / magnitude;
        let unit_direction_long = direction_long / magnitude;

        let new_lat = current_lat + unit_direction_lat * speed;
        let new_long = current_long + unit_direction_long * speed;

        (new_lat, new_long)
    }

    /// Actualiza la location del Drone en el broker.
    /// Envia un Publish configurado segun el archivo de config
    /// en el que por payload se tiene la location del Drone junto con su ID
    fn update_location(&mut self) {
        let publish_config = match PublishConfig::read_config(
            "src/drones/publish_config.json",
            PayloadTypes::DroneLocation(self.id, self.location),
        ) {
            Ok(config) => config,
            Err(e) => {
                println!("Error reading publish config: {:?}", e);
                return;
            }
        };

        let lock = match self.send_to_client_channel.lock() {
            Ok(locked) => locked,
            Err(e) => {
                println!("Error locking send to client channel: {:?}", e);
                return;
            }
        };
        if let Some(ref sender) = *lock {
            match sender.send(Box::new(publish_config)) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error sending to client channel: {:?}", e);
                }
            };
        }
    }

    /// Publica un mensaje de tipo attendingincident en el broker.
    /// Envia un Publish configurado segun el archivo de config
    /// en el que por payload se tiene el incidente que el Drone esta atendiendo
    /// junto con su ID
    fn publish_attending_accident(&mut self, location: Location) {
        let incident = Incident::new(location);
        let incident_payload = IncidentPayload::new(incident.clone());
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

        let lock = match self.send_to_client_channel.lock() {
            Ok(locked) => locked,
            Err(e) => {
                println!("Error locking send to client channel: {:?}", e);
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
    /// Actualiza la location del Drone segun la formula de la circunferencia
    /// Esta funcion se utiliza para calcular el movimiento del dron en estado pasivo
    fn update_target_location(&mut self) -> Result<(), DroneError> {
        let current_time = Utc::now().timestamp_millis() as f64;
        let angle = ((current_time * ANGLE_SCALING_FACTOR) / MILISECONDS_PER_SECOND) % TWO_PI;
        let operation_radius = self.drone_config.get_operation_radius();
        let new_target_lat = self.center_location.lat + operation_radius * angle.cos();
        let new_target_long = self.center_location.long + operation_radius * angle.sin();
        self.target_location = Location::new(new_target_lat, new_target_long);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        mqtt::broker::Broker,
        utils::location::{self, Location},
    };
    use std::{
        sync::{Arc, Condvar, Mutex},
        thread,
    };

    use super::*;

    #[test]
    fn test_01_reading_connect_config_ok() -> Result<(), DroneError> {
        let file_path = "./src/drones/connect_config.json";
        let id = 1;
        let connect_config = Drone::read_connect_config(file_path, id);

        match connect_config {
            Ok(config) => {
                println!("Connect config: {:?}", config);

                Ok(())
            }
            Err(e) => return Err(e),
        }
    }

    #[test]
    fn test_02_drone_arranca_en_waiting_state() {
        let args = vec!["127.0.0.1".to_string(), "5001".to_string()];
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

        let handle = thread::spawn(move || {
            let latitude = 0.0;
            let longitude = 0.0;
            let location = location::Location::new(latitude, longitude);
            let center_location = location::Location::new(0.0, 0.0);
            let mut drone = Drone::new(
                1,
                location,
                center_location,
                "./src/drones/drone_config.json",
                "127.0.0.1:5001".to_string(),
            )
            .unwrap();

            let _ = drone.run_drone();

            assert_eq!(drone.get_state(), DroneState::Waiting);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_03_get_id_ok() {
        let args = vec!["127.0.0.1".to_string(), "5003".to_string()];
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

        let handle = thread::spawn(move || {
            let latitude = 0.0;
            let longitude = 0.0;
            let location = location::Location::new(latitude, longitude);
            let center_location = location::Location::new(0.0, 0.0);
            let drone = Drone::new(
                1,
                location,
                center_location,
                "./tests/drone_config_test.json",
                "127.0.0.1:5003".to_string(),
            )
            .unwrap();

            assert_eq!(drone.get_id(), 1);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_04_drone_bad_config_file() {
        let latitude = 0.0;
        let longitude = 0.0;
        let location = location::Location::new(latitude, longitude);
        let center_location = location::Location::new(0.0, 0.0);
        let drone = Drone::new(
            1,
            location,
            center_location,
            "./tests/bad_config_file.json",
            "127.0.0.1:5000".to_string(),
        );
        assert!(drone.is_err());
    }

    #[test]
    fn test_drone_stays_within_radius() {
        let args = vec!["127.0.0.1".to_string(), "5004".to_string()];
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

        let handle = thread::spawn(move || {
            let latitude = 0.0;
            let longitude = 0.0;
            let location = location::Location::new(latitude, longitude);
            let center_location = location::Location::new(0.0, 0.0);
            let mut drone = Drone::new(
                1,
                location,
                center_location,
                "./src/drones/drone_config.json",
                "127.0.0.1:5004".to_string(),
            )
            .unwrap();
            let target_location = location::Location::new(0.001, 0.001);
            let radius = 0.005;
            let _ = drone.drone_movement(target_location);

            let distance_from_center =
                ((drone.location.lat).powi(2) + (drone.location.long).powi(2)).sqrt();
            assert!(
                distance_from_center <= radius,
                "Drone should stay within the radius."
            );
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_new_drone() {
        let args = vec!["127.0.0.1".to_string(), "5007".to_string()];
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
        let handle = thread::spawn(move || {
            let location = Location {
                lat: 0.0,
                long: 0.0,
            };
            let center_location = Location {
                lat: 0.0,
                long: 0.0,
            };
            let config_file_path = "./src/drones/drone_config.json";
            let address = "127.0.0.1:5007".to_string();

            let drone: Result<Drone, DroneError> =
                Drone::new(1, location, center_location, config_file_path, address);

            assert!(drone.is_ok());
            let drone = drone.unwrap();
            assert_eq!(drone.id, 1);
            assert_eq!(drone.location, location);
            assert_eq!(drone.center_location, center_location);
            assert_eq!(drone.battery_level, 100);
            assert_eq!(drone.drone_state, DroneState::Waiting);
        });
        handle.join().unwrap();
    }

    #[test]
    fn test_battery_discharge() {
        let args = vec!["127.0.0.1".to_string(), "5008".to_string()];
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
        let mut drone = setup_test_drone("127.0.0.1:5008".to_string());
        for _ in 0..100 {
            drone.update_battery_discharge(Utc::now(), Utc::now() + chrono::Duration::seconds(120));
        }

        assert!(drone.battery_level <= 0);
        assert_eq!(drone.drone_state, DroneState::LowBatteryLevel);
    }

    #[test]
    fn test_charge_battery() {
        let args = vec!["127.0.0.1".to_string(), "5009".to_string()];
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

        let mut drone = setup_test_drone("127.0.0.1:5009".to_string());
        for _ in 0..100 {
            drone.update_battery_discharge(Utc::now(), Utc::now() + chrono::Duration::seconds(120));
        }

        assert!(drone.battery_level <= 0);
        assert_eq!(drone.drone_state, DroneState::LowBatteryLevel);

        for _ in 0..100 {
            drone.update_battery_charge(Utc::now(), Utc::now() + chrono::Duration::seconds(120));
        }
        assert_eq!(drone.battery_level, 100);
        assert_eq!(drone.get_state(), DroneState::Waiting);
    }

    #[test]
    fn test_drone_movement() {
        let args = vec!["127.0.0.1".to_string(), "5010".to_string()];
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
        let handle = thread::spawn(move || {
            let mut drone = setup_test_drone("127.0.0.1:5010".to_string());
            drone.location = Location {
                lat: 0.0,
                long: 0.0,
            };
            let target_location = Location {
                lat: 1.0,
                long: 1.0,
            };
            let drone_arc = Arc::new(Mutex::new(drone));
            for _ in 0..14 {
                let mut drone = drone_arc.lock().unwrap();
                drone.update_drone_position(target_location).unwrap();
            }
            let drone = drone_arc.lock().unwrap();
            let new_location = &drone.location;
            println!("New location: {:?}", new_location);
            assert!(new_location.lat > 0.0 && new_location.lat < 1.0);
            assert!(new_location.long > 0.0 && new_location.long < 1.0);
        });
        handle.join().unwrap();
    }

    // #[test]
    // fn drone_con_poca_bateria_va_a_cargarse() {
    //     let args = vec!["127.0.0.1".to_string(), "5020".to_string()];
    //     let mut broker = match Broker::new(args) {
    //         Ok(broker) => broker,
    //         Err(e) => panic!("Error creating broker: {:?}", e),
    //     };

    //     let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
    //     let server_ready_clone = server_ready.clone();
    //     thread::spawn(move || {
    //         {
    //             let (lock, cvar) = &*server_ready_clone;
    //             let mut ready = lock.lock().unwrap();
    //             *ready = true;
    //             cvar.notify_all();
    //         }
    //         let _ = broker.server_run();
    //     });

    //     // Wait for the server to start
    //     {
    //         let (lock, cvar) = &*server_ready;
    //         let mut ready = lock.lock().unwrap();
    //         while !*ready {
    //             ready = cvar.wait(ready).unwrap();
    //         }
    //     }
    //     let handle = thread::spawn(move || {
    //         let mut drone = setup_test_drone("127.0.0.1:5020".to_string());
    //         drone.center_location = Location {
    //             lat: 2.0,
    //             long: 1.0,
    //         };
    //         drone.location = Location {
    //             lat: 0.0,
    //             long: 0.0,
    //         };
    //         let _target_location = Location {
    //             lat: 1.0,
    //             long: 1.0,
    //         };
    //         drone.drone_state = DroneState::LowBatteryLevel;
    //         drone.battery_level = 19;
    //         let drone_arc = Arc::new(Mutex::new(drone));
    //         let mut se_cargo = false;
    //         for _ in 0..100 {
    //             let mut drone = drone_arc.lock().unwrap();
    //             drone.patrolling_in_operating_radius().unwrap();
    //             if drone.location == drone.center_location {
    //                 se_cargo = true;
    //                 break;
    //             }
    //         }
    //         assert!(se_cargo);
    //     });
    //     handle.join().unwrap();
    // }

    // #[test]
    // fn test_drone_movement_out_of_bounds() {
    //     let args = vec!["127.0.0.1".to_string(), "5011".to_string()];
    //     let mut broker = match Broker::new(args) {
    //         Ok(broker) => broker,
    //         Err(e) => panic!("Error creating broker: {:?}", e),
    //     };

    //     let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
    //     let server_ready_clone = server_ready.clone();
    //     thread::spawn(move || {
    //         {
    //             let (lock, cvar) = &*server_ready_clone;
    //             let mut ready = lock.lock().unwrap();
    //             *ready = true;
    //             cvar.notify_all();
    //         }
    //         let _ = broker.server_run();
    //     });

    //     // Wait for the server to start
    //     {
    //         let (lock, cvar) = &*server_ready;
    //         let mut ready = lock.lock().unwrap();
    //         while !*ready {
    //             ready = cvar.wait(ready).unwrap();
    //         }
    //     }
    //     let handle = thread::spawn(move || {
    //         let mut drone = setup_test_drone("127.0.0.1:5011".to_string());
    //         let drone_arc = Arc::new(Mutex::new(drone.clone()));
    //         drone.location = Location {
    //             lat: 0.0,
    //             long: 0.0,
    //         };
    //         let target_location = Location {
    //             lat: 1.0,
    //             long: 1.0,
    //         };
    //         for _ in 0..14 {
    //             let mut drone = drone_arc.lock().unwrap();
    //             drone
    //                 .update_drone_position_and_battery(&target_location)
    //                 .unwrap();
    //         }

    //         let new_location = drone.location;

    //         let distance_from_center =
    //             ((new_location.lat - 0.0).powi(2) + (new_location.long - 0.0).powi(2)).sqrt();
    //         assert!(distance_from_center <= 0.005);
    //     });
    //     handle.join().unwrap();
    // }

    /// Esta función de prueba simula el comportamiento de un dron cuando su batería se descarga.
    ///
    /// La prueba comienza creando un nuevo broker y ejecutándolo en un hilo separado.
    /// Luego, crea un dron en una ubicación específica e inicia un hilo para simular la descarga de la batería.
    ///
    /// En otro hilo, el dron se mueve hacia una ubicación objetivo hasta que su nivel de batería es bajo.
    /// Cuando el nivel de batería del dron es bajo, verifica que la ubicación del dron no sea la misma que la ubicación objetivo.
    /// Esto se debe a que el dron deberia estar en camino o yendo hacia el centro de carga debido a su nivel de bateria
    ///
    /// Después de que el nivel de batería del dron es bajo, deja de mover el dron y verifica que la ubicación del dron sea la misma que la ubicación objetivo.
    /// Esto se debe a que el dron debería haber regresado a la ubicación objetivo después de que su batería se descargó.
    ///
    // #[test]
    // fn test_volver_a_pos_original_despues_de_cargarse() {
    //     let args = vec!["127.0.0.1".to_string(), "5012".to_string()];
    //     let mut broker = match Broker::new(args) {
    //         Ok(broker) => broker,
    //         Err(e) => panic!("Error creating broker: {:?}", e),
    //     };

    //     let server_ready = Arc::new((Mutex::new(false), Condvar::new()));
    //     let server_ready_clone = server_ready.clone();
    //     thread::spawn(move || {
    //         {
    //             let (lock, cvar) = &*server_ready_clone;
    //             let mut ready = lock.lock().unwrap();
    //             *ready = true;
    //             cvar.notify_all();
    //         }
    //         let _ = broker.server_run();
    //     });

    //     // Wait for the server to start
    //     {
    //         let (lock, cvar) = &*server_ready;
    //         let mut ready = lock.lock().unwrap();
    //         while !*ready {
    //             ready = cvar.wait(ready).unwrap();
    //         }
    //     }
    //     let handle = thread::spawn(move || {
    //         let latitude = 0.0;
    //         let longitude = 0.0;
    //         let location = location::Location::new(latitude, longitude);
    //         let center_location = location::Location::new(0.0, 0.0);
    //         let drone = Drone::new(
    //             1,
    //             location,
    //             center_location,
    //             "./src/drones/drone_config.json",
    //             "127.0.0.1:5012".to_string(),
    //         )
    //         .unwrap();

    //         //thread de descarga de bateria
    //         let mut drone_clone = drone.clone();
    //         let _ = thread::spawn(move || {
    //             let _ = drone_clone.battery_discharge();
    //         });

    //         let mut drone_clone = drone.clone();
    //         let drone_clone2 = drone.clone();
    //         let _ = thread::spawn(move || {
    //             let mut terminado = false;
    //             let target_location = location::Location::new(0.001, 0.001);
    //             let _ = drone.location.clone();

    //             while !terminado {
    //                 let target_location = location::Location::new(0.001, 0.001);
    //                 let _ = drone_clone.location.clone();

    //                 let _ = drone_clone.drone_movement(target_location.clone());
    //                 if drone_clone2.clone().get_state() == DroneState::LowBatteryLevel {
    //                     assert_ne!(drone_clone.location, target_location.clone());
    //                     terminado = true;
    //                 }
    //             }
    //             assert_eq!(drone_clone.location, target_location.clone());
    //         });
    //     });
    //     handle.join().unwrap();
    // }

    // Helper function to setup a test drone
    fn setup_test_drone(addres: String) -> Drone {
        let location = Location {
            lat: 0.0,
            long: 0.0,
        };
        let center_location = Location {
            lat: 0.0,
            long: 0.0,
        };
        let config_file_path = "./src/drones/drone_config.json";
        let address = addres.to_string();

        Drone::new(1, location, center_location, config_file_path, address).unwrap()
    }
}
