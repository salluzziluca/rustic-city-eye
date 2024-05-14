use std::env::args;

use rustic_city_eye::mqtt::{neobroker::Broker, protocol_error::ProtocolError};

fn main() -> Result<(), ProtocolError> {
    let argv = args().collect::<Vec<String>>();
    let mut broker = Broker::new(argv)?;
    let _ = broker.server_run();
    Ok(())
}
