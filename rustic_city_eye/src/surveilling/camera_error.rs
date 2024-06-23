use std::fmt;

///Here are detailed all the errors that the protocol is capable of throwing.
/// Unspecified se usa de placeholder para los results de los tests
#[derive(PartialEq)]
#[derive(Debug)]
pub enum CameraError {
    SendError,
}

impl fmt::Display for CameraError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CameraError::SendError => {
                write!(f, "Error sending message via channel")
            }
        }
    }
}
