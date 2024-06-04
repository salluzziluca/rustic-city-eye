#[derive(Debug)]
pub struct ClientError {
    message: String,
}

impl ClientError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ClientError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client_error() {
        let error = ClientError::new("error message");
        assert_eq!(error.to_string(), "error message");
    }
}
