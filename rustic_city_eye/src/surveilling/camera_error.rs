use std::fmt;

///Here are detailed all the errors that the protocol is capable of throwing.
/// Unspecified se usa de placeholder para los results de los tests
#[derive(PartialEq, Debug)]
pub enum CameraError {
    SendError,
    WriteError,
    ArcMutexError(String),
    DeleteDirError(String),
    WatcherError(String),
}

impl fmt::Display for CameraError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CameraError::SendError => {
                write!(f, "Error sending message via channel")
            }
            CameraError::WriteError => {
                write!(f, "Error writing to stream")
            }
            CameraError::ArcMutexError(e) => {
                write!(f, "Error with Arc Mutex: {}", e)
            }
            CameraError::DeleteDirError(e) => {
                write!(f, "Error deleting directory: {}", e)
            }
            CameraError::WatcherError(e) => {
                write!(f, "Error with watcher: {}", e)
            }
        }
    }
}
