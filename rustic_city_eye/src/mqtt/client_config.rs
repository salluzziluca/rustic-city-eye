use std::fs::File;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct ClientConfig {
    pub client_id: String,
    pub state: bool,
    pub subscriptions: Vec<String>,
}

impl ClientConfig {
    pub fn new(client_id: String) -> Self {
        let state = true;
        Self {
            client_id,
            state,
            subscriptions: Vec::new(),
        }
    }
}
