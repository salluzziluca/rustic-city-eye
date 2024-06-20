pub enum CameraError {
    SendError,
}

impl std::fmt::Debug for CameraError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CameraError::SendError => {
                write!(f, "Error sending message via channel")
            }
        }
    }
}
