use std::env::args;

use rustic_city_eye::mqtt::{broker::Broker, protocol_error::ProtocolError};

fn main() -> Result<(), ProtocolError> {
    let argv = args().collect::<Vec<String>>();
    let broker = Broker::new(argv)?;
    let _ = broker.server_run();
    Ok(())
}