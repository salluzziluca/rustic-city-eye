use std::{env::args, io::stdin};

use rustic_city_eye::{camera_system::camera::Camera, mqtt::protocol_error::ProtocolError};

fn main() -> Result<(), ProtocolError> {
    let argv = args().collect::<Vec<String>>();

    let mut camera = Camera::new(argv)?;
    let _ = camera.app_run(&mut stdin().lock());
    Ok(())
}