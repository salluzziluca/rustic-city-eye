use std::{env::args, io::stdin};

use rustic_city_eye::{camera_system::camera_system::CameraSystem, mqtt::protocol_error::ProtocolError};

fn main() -> Result<(), ProtocolError> {
    let argv = args().collect::<Vec<String>>();

    let mut camera_system = CameraSystem::new(argv)?;
    let _ = camera_system.app_run(&mut stdin().lock());
    Ok(())
}
