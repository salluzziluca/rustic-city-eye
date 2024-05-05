//! Se conecta mediante TCP a la dirección asignada por argv.
//! Lee lineas desde stdin y las manda mediante el socket.

use rustic_city_eye::camera_system::camera::Camera;
use rustic_city_eye::camera_system::camera_system::CameraSystem;
use rustic_city_eye::mqtt::client::Client;
use rustic_city_eye::mqtt::connect_properties;
use rustic_city_eye::mqtt::protocol_error::ProtocolError;
use rustic_city_eye::mqtt::will_properties;

use std::env::args;
use std::io::stdin;
use std::io::{BufRead, Error};
use std::io::{BufReader, Read};

pub struct MonitoringApp {
    monitoring_app_client: Client,
    camera_system: CameraSystem,
}

fn main() -> Result<(), ProtocolError> {
    let argv = args().collect::<Vec<String>>();

    let mut monitoring_app = MonitoringApp::new(argv)?;
    let _ = monitoring_app.app_run(&mut stdin().lock());
    Ok(())
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

        let mut monitoring_app = MonitoringApp {
            camera_system: CameraSystem::build(args.clone())?,
            monitoring_app_client: match Client::build(
                args,
                will_properties,
                connect_properties,
                true,
                true,
                1,
                true,
                "prueba".to_string(),
                "".to_string(),
                35,
                "kvtr33".to_string(),
                "camera_system".to_string(),
                "soy el camera_system y me desconecté".to_string(),
            ) {
                Ok(client) => client,
                Err(err) => return Err(err)
            },
        };
        let camera1 = Camera::new();
        monitoring_app.camera_system.add_camera(camera1);
        Ok(monitoring_app)
    }

    /// En este momento la app de monitoreo ya tiene todos los clientes conectados y funcionales
    /// Aca es donde se gestiona la interaccion entre los diferentes clientes
    pub fn app_run(&mut self, stream: &mut dyn Read) -> Result<(), Error> {
        let reader = BufReader::new(stream);

        for line in reader.lines() {
            if let Ok(line) = line {
                if line.starts_with("publish:") {
                    let (_, post_colon) = line.split_at(8); // "publish:" is 8 characters
                    let message = post_colon.trim(); // remove leading/trailing whitespace
                    println!("Publishing message: {}", message);
                    self.monitoring_app_client.publish_message(message);
                } else if line.starts_with("subscribe:") {
                    let (_, post_colon) = line.split_at(10); // "subscribe:" is 10 characters
                    let topic = post_colon.trim(); // remove leading/trailing whitespace
                    println!("Subscribing to topic: {}", topic);
                    self.monitoring_app_client.subscribe(topic);

                } else {
                    println!("Comando no reconocido: {}", line);
                }
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Error al leer linea",
                ));
            }
        }
        Ok(())
    }
}
