use std::fmt;

///Here are detailed all the errors that the protocol is capable of throwing.
#[derive(Debug)]
pub enum ProtocolError {
    ConectionError,
    InvalidQOS,
    InvalidNumberOfArguments
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProtocolError::ConectionError => write!(f, "Error while connecting to broker."),
            ProtocolError::InvalidQOS => write!(f, "Error: Invalid QoS value. It must be 0 or 1."),
            ProtocolError::InvalidNumberOfArguments => write!(f, "Error: Invalid number of arguments"),

        }
    }
}
