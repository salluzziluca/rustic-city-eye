use std::env::args;

use broker::broker::Broker;
use utils::protocol_error::ProtocolError;

fn main() -> Result<(), ProtocolError> {
    let argv = args().collect::<Vec<String>>();
    let mut broker = Broker::new(argv)?;
    match broker.server_run() {
        Ok(_) => {}
        Err(e) => return Err(e),
    }
    Ok(())
}
