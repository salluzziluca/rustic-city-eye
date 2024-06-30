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

    pub fn change_client_state(client_id: String, state: bool) {
        // cambia el estado de un cliente en el archivo json
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        let file = std::fs::File::open(path.clone()).unwrap();
        let client_config: ClientConfig = serde_json::from_reader(file).unwrap();
        let new_client_config = ClientConfig {
            client_id: client_config.client_id,
            state,
            subscriptions: client_config.subscriptions,
        };
        let json = serde_json::to_string(&new_client_config).unwrap();
        let _ = std::fs::write(path, json);
    }

    pub fn save_client_log_in_json(client_id: String) {
        // guarda en un archivo json el log de los clientes, con la estructura client_config

        let client_config = ClientConfig::new(client_id.clone());
        let json = serde_json::to_string(&client_config).unwrap();
        let path = format!("./src/mqtt/clients/{}.json", client_id);

        let _ = std::fs::write(path, json);
    }

    pub fn add_new_subscription(client_id: String, topic: String) {
        // agrega una nueva suscripción a un cliente en el archivo json
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        let file = std::fs::File::open(path.clone()).unwrap();
        let mut client_config: ClientConfig = serde_json::from_reader(file).unwrap();
        client_config.subscriptions.push(topic);
        let json = serde_json::to_string(&client_config).unwrap();
        let _ = std::fs::write(path, json);
    }

    pub fn remove_subscription(client_id: String, topic: String) {
        // remueve una suscripción de un cliente en el archivo json
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        let file = std::fs::File::open(path.clone()).unwrap();
        let mut client_config: ClientConfig = serde_json::from_reader(file).unwrap();
        let index = client_config.subscriptions.iter().position(|x| x == &topic).unwrap();
        client_config.subscriptions.remove(index);
        let json = serde_json::to_string(&client_config).unwrap();
        let _ = std::fs::write(path, json);
    }

    pub fn _client_exists(client_id: String) -> bool {
        // verifica si un cliente existe en el archivo json
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        std::fs::metadata(path).is_ok()
    }
}
