use std::env::args;

use rustic_city_eye::{
    monitoring_app::monitoring_app::MonitoringApp, mqtt::protocol_error::ProtocolError,
};

fn main() -> Result<(), ProtocolError> {
    let argv = args().collect::<Vec<String>>();

    let mut monitoring_app = MonitoringApp::new(argv)?;

    let _ = monitoring_app.app_run();
    Ok(())
}
