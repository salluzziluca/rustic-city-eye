//! Se conecta mediante TCP a la dirección asignada por argv.
//! Lee lineas desde stdin y las manda mediante el socket.

//use crate::camera_system::camera_system::CameraSystem;
use crate::mqtt::client::Client;
use crate::mqtt::connect_properties;
use crate::mqtt::protocol_error::ProtocolError;
use crate::mqtt::will_properties;

use std::{
    io::{stdin, BufRead, Error, ErrorKind},
    sync::mpsc,
};

pub struct MonitoringApp {
    monitoring_app_client: Client,
    // camera_system: CameraSystem,
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

        //let camera_system = CameraSystem::new(args.clone())?;

        let monitoring_app = MonitoringApp {
            // camera_system,
            monitoring_app_client: match Client::new(
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
                Err(err) => return Err(err),
            },
        };
        Ok(monitoring_app)
    }

    /// En este momento la app de monitoreo ya tiene todos los clientes conectados y funcionales
    /// Aca es donde se gestiona la interaccion entre los diferentes clientes
    pub fn app_run(&mut self) -> Result<(), Error> {
        let (tx, rx) = mpsc::channel();
        let _ = self.monitoring_app_client.client_run(rx);

        let stdin = stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => {
                    tx.send(line).map_err(|err| {
                        Error::new(ErrorKind::Other, format!("Failed to send line: {}", err))
                    })?;
                }
                Err(_) => println!("error in line"),
            }
        }

        Ok(())
    }

    // pub fn add_camera (&mut self) -> Result<(), ProtocolError> {
    //     self.camera_system.add_camera()?;

    //     Ok(())
    // }
}
