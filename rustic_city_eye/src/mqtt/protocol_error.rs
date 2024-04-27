use std::fmt;

///Here are detailed all the errors that the protocol is capable of throwing.
#[derive(Debug)]
pub enum ProtocolError {
    ConectionError
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProtocolError::ConectionError => write!(f, "Error while connecting to broker."),

        }
    }
}