use std::{
    env::args,
    io::{stdin, Read},
};

use rustic_city_eye::{
    monitoring_app::monitoring_app::MonitoringApp, mqtt::protocol_error::ProtocolError,
};

fn main() -> Result<(), ProtocolError> {
    let argv = args().collect::<Vec<String>>();

    let mut monitoring_app = MonitoringApp::new(argv)?;
    let stream: Box<dyn Read + Send> = Box::new(stdin());

    let _ = monitoring_app.app_run(stream);
    Ok(())
}
