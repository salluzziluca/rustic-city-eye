use std::{
    env::args,
    io::{stdin, Read},
};

use rustic_city_eye::{
    mqtt::protocol_error::ProtocolError, surveilling::camera_system::CameraSystem,
};

fn main() -> Result<(), ProtocolError> {
    let argv = args().collect::<Vec<String>>();

    let mut _camera_system = CameraSystem::new(argv)?;
    let _stream: Box<dyn Read + Send> = Box::new(stdin());

    // let _ = camera_system.app_run(stream);
    Ok(())
}
