// use std::{
//     env::args,
//     io::{stdin, Read},
// };

// use rustic_city_eye::{camera_system::camera::Camera, mqtt::protocol_error::ProtocolError};

// fn main() -> Result<(), ProtocolError> {
//     let argv = args().collect::<Vec<String>>();

//     let mut camera = Camera::new(argv)?;
//     let stream: Box<dyn Read + Send> = Box::new(stdin());

//     let _ = camera.camera_run(stream);
//     Ok(())
// }
