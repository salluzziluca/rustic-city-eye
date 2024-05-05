//! Se conecta mediante TCP a la dirección asignada por argv.
//! Lee lineas desde stdin y las manda mediante el socket.

use rustic_city_eye::camera_system::camera::Camera;
use rustic_city_eye::camera_system::camera_system::CameraSystem;
use rustic_city_eye::mqtt::client::Client;
use rustic_city_eye::mqtt::connect_properties;
use rustic_city_eye::mqtt::will_properties;

use std::env::args;
use std::io::stdin;
use std::io::Error;
use std::io::{BufRead, BufReader, Read};

static CLIENT_ARGS: usize = 3;

pub struct MonitoringApp {
    camera_system: CameraSystem,
}

fn main() -> Result<(), std::io::Error> {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != CLIENT_ARGS {
        println!("Cantidad de argumentos inválido");
        let app_name = &argv[0];
        println!("{:?} <host> <puerto>", app_name);
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid argument count",
        ));
    }

    let address = argv[1].clone() + ":" + &argv[2];
    println!("Soy la app de Monitoreo, conectándome AAAA {:?}", address);
    let mut monitoring_app = MonitoringApp::new();
    monitoring_app.client_run(&address, &mut stdin().lock())?;
    Ok(())
}

#[allow(dead_code)]
impl MonitoringApp {
    pub fn new() -> MonitoringApp {
        let mut monitoring_app = MonitoringApp {
            camera_system: CameraSystem::new(),
        };
        let camera1 = Camera::new();
        monitoring_app.camera_system.add_camera(camera1);
        monitoring_app
    }
    /// Client run recibe una dirección y cualquier cosa "legible"
    /// Crea un cliente del tipo app de monitoreo, crea un cliente del tipo camera system y crea varios clientes del tipo camera que se conectan al camera system.
    pub fn client_run(&mut self, address: &str, stream: &mut dyn Read) -> Result<(), Error> {
        let reader = BufReader::new(stream);
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

        let mut client = match Client::new(
            address,
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
            "topic".to_string(),
            "chauchis".to_string(),
        ) {
            Ok(client) => client,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Error al crear cliente",
                ))
            } //todo
        };

        // for line in reader.lines() {
        //     if let Ok(line) = line {
        //         if line.starts_with("publish:") {
        //             let (_, post_colon) = line.split_at(8); // "publish:" is 8 characters
        //             let message = post_colon.trim(); // remove leading/trailing whitespace
        //             println!("Publishing message: {}", message);
        //             client.publish_message(message);
        //         } else if line.starts_with("subscribe:") {
        //             let (_, post_colon) = line.split_at(10); // "subscribe:" is 10 characters
        //             let topic = post_colon.trim(); // remove leading/trailing whitespace
        //             println!("Subscribing to topic: {}", topic);
        //             client.subscribe(topic);
        //         } else {
        //             println!("Comando no reconocido: {}", line);
        //         }
        //     } else {
        //         return Err(std::io::Error::new(
        //             std::io::ErrorKind::Other,
        //             "Error al leer linea",
        //         ));
        //     }
        // }

        Ok(())
    }
}
