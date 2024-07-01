use std::fs::File;

use serde::{Deserialize, Serialize};

/// Estructura que representa la configuración de un cliente
#[derive(Serialize, Deserialize)]
pub(crate) struct ClientConfig {
    /// Identificador del cliente
    pub client_id: String,
    /// Estado del cliente
    pub state: bool,
    /// Lista de suscripciones del cliente
    pub subscriptions: Vec<String>,
}

/// Implementación de métodos para la estructura ClientConfig
/// Métodos para guardar, modificar y obtener la configuración de un cliente
impl ClientConfig {
    /// Crea una nueva configuración de cliente
    pub fn new(client_id: String) -> Self {
        let state = true;
        Self {
            client_id,
            state,
            subscriptions: Vec::new(),
        }
    }
  
    /// Cambia el estado de un cliente en el archivo json
    pub fn change_client_state(
        client_id: String,
        state: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        let file = File::open(&path)?;
        let client_config: ClientConfig = serde_json::from_reader(file)?;
        let new_client_config = ClientConfig {
            client_id: client_config.client_id,
            state,
            subscriptions: client_config.subscriptions,
        };
        let json = serde_json::to_string(&new_client_config)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Guarda la configuración de un cliente en un archivo json
    pub fn save_client_log_in_json(client_id: String) -> Result<(), Box<dyn std::error::Error>> {

        let client_config = ClientConfig::new(client_id.clone());
        let json = serde_json::to_string(&client_config)?;
        let path = format!("./src/mqtt/clients/{}.json", client_id);

        std::fs::write(path, json)?;
        Ok(())
    }

    /// Agrega una nueva suscripción a un cliente en el archivo json
    pub fn add_new_subscription(
        client_id: String,
        topic: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        if !ClientConfig::client_exists(client_id.clone()) {
            ClientConfig::save_client_log_in_json(client_id.clone())?;
        }
        let file = std::fs::File::open(path.clone())?;
        let mut client_config: ClientConfig = serde_json::from_reader(file)?;
        client_config.subscriptions.push(topic);
        let json = serde_json::to_string(&client_config)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Remueve una suscripción de un cliente en el archivo json
    pub fn remove_subscription(
        client_id: String,
        topic: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        let file = std::fs::File::open(path.clone())?;
        let mut client_config: ClientConfig = serde_json::from_reader(file)?;
        if let Some(index) = client_config.subscriptions.iter().position(|x| x == &topic) {
            client_config.subscriptions.remove(index);
            let json = serde_json::to_string(&client_config)?;
            std::fs::write(path, json)?;
        } else {
            return Err("Subscription not found".into());
        }
        Ok(())
    }

    /// Verifica si un cliente existe en el archivo json
    pub fn client_exists(client_id: String) -> bool {
        // verifica si un cliente existe en el archivo json
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        std::fs::metadata(path).is_ok()
    }

    /// Obtiene un cliente del archivo json
    pub fn _get_client(client_id: String) -> ClientConfig {
        // obtiene un cliente del archivo json
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        let file = std::fs::File::open(path).unwrap();
        serde_json::from_reader(file).unwrap()
    }

    /// Remueve un cliente del archivo json
    pub fn remove_client(client_id: String) {
        // remueve un cliente del archivo json
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client_config() {
        let client_id = "test".to_string();
        let client_config = ClientConfig::new(client_id.clone());
        assert_eq!(client_config.client_id, client_id);
        assert_eq!(client_config.state, true);
        assert_eq!(client_config.subscriptions.len(), 0);
    }

    #[test]
    fn test_change_client_state() {
        let client_id = "test".to_string();
        let _ = ClientConfig::save_client_log_in_json(client_id.clone());
        let _ = ClientConfig::change_client_state(client_id.clone(), false);
        let client_config = ClientConfig::_get_client(client_id.clone());
        assert_eq!(client_config.state, false);
        ClientConfig::remove_client(client_id.clone());
    }

    #[test]
    fn test_save_client_log_in_json() {
        let client_id = "test".to_string();
        let _ = ClientConfig::save_client_log_in_json(client_id.clone());
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        assert!(std::fs::metadata(path).is_ok());
        ClientConfig::remove_client(client_id.clone());
    }

    #[test]
    fn test_add_new_subscription() {
        let client_id = "test".to_string();
        let topic = "test".to_string();
        let _ = ClientConfig::save_client_log_in_json(client_id.clone());
        let _ = ClientConfig::add_new_subscription(client_id.clone(), topic.clone());
        let client_config = ClientConfig::_get_client(client_id.clone());
        assert_eq!(client_config.subscriptions[0], topic);
        ClientConfig::remove_client(client_id.clone());
    }

    #[test]
    fn test_remove_subscription() {
        let client_id = "test".to_string();
        let topic = "test".to_string();
        let _ = ClientConfig::save_client_log_in_json(client_id.clone());
        let _ = ClientConfig::add_new_subscription(client_id.clone(), topic.clone());
        let _ = ClientConfig::remove_subscription(client_id.clone(), topic.clone());
        let client_config = ClientConfig::_get_client(client_id.clone());
        assert_eq!(client_config.subscriptions.len(), 0);
        ClientConfig::remove_client(client_id.clone());
    }
}
