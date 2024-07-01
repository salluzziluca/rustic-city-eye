use std::env::args;

use rustic_city_eye::mqtt::{broker::Broker, protocol_error::ProtocolError};

fn main() -> Result<(), ProtocolError> {
    let argv = args().collect::<Vec<String>>();
    let mut broker = Broker::new(argv)?;
    match broker.server_run() {
        Ok(_) => {}
        Err(e) => {
            panic!("Error running broker: {:?}", e);
        }
    }
    Ok(())
}
