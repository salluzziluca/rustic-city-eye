use std::fs::File;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct ClientConfig {
    client_id: String,
    subscriptions: Vec<String>,
}

impl ClientConfig {
    pub fn new(client_id: String) -> Self {
        Self {
            client_id,
            subscriptions: Vec::new(),
        }
    }
}
