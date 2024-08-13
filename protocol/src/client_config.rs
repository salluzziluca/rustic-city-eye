use std::fs::File;

use serde::{Deserialize, Serialize};

use utils::protocol_error::ProtocolError;
use super::client_message::ClientMessage;

/// Estructura que representa la configuración de un cliente
#[derive(Serialize, Deserialize)]
pub struct ClientConfig {
    /// Identificador del cliente
    pub client_id: String,
    /// Estado del cliente
    pub state: bool,
    /// Lista de suscripciones del cliente
    pub subscriptions: Vec<String>,
    /// Lista de mensajes pendientes
    pub pending_messages: Vec<ClientMessage>,
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
            pending_messages: Vec::new(),
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
            pending_messages: client_config.pending_messages,
        };
        let json = serde_json::to_string(&new_client_config)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Guarda la configuración de un cliente en un archivo json
    pub fn create_client_log_in_json(client_id: String) -> Result<(), Box<dyn std::error::Error>> {
        let client_config = ClientConfig::new(client_id.clone());
        let json = serde_json::to_string(&client_config)?;
        let path = format!("./broker/src/clients/{}.json", client_id);

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
            let _ = ClientConfig::create_client_log_in_json(client_id.clone());
        }

        let file = std::fs::File::open(path.clone())?;
        let mut client_config: ClientConfig = serde_json::from_reader(file)?;
        if client_config.subscriptions.contains(&topic) {
            return Ok(());
        }

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

    /// Remueve un cliente del archivo json
    pub fn delete_client_file(client_id: String) -> Result<(), ProtocolError> {
        // remueve un cliente del archivo json
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        if std::fs::metadata(&path).is_err() {
            return Ok(());
        }

        match std::fs::remove_file(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(ProtocolError::RemoveClientError(e.to_string())),
        }
    }

    pub fn client_is_online(client_id: String) -> bool {
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        if std::fs::metadata(&path).is_ok() {
            let file = match std::fs::File::open(path) {
                Ok(file) => file,
                Err(_) => return false,
            };
            let client_config: ClientConfig = match serde_json::from_reader(file) {
                Ok(client_config) => client_config,
                Err(_) => return false,
            };
            client_config.state
        } else {
            false
        }
    }

    pub fn _remove_all_subscriptions_from_file(
        client_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        let file = std::fs::File::open(path.clone())?;
        let mut client_config: ClientConfig = serde_json::from_reader(file)?;
        client_config.subscriptions = Vec::new();
        let json = serde_json::to_string(&client_config)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn _get_client_subscriptions(
        client_id: String,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        let file = std::fs::File::open(path.clone())?;
        let client_config: ClientConfig = serde_json::from_reader(file)?;
        Ok(client_config.subscriptions)
    }

    pub fn add_offline_message(
        client_id: String,
        message: ClientMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        if !ClientConfig::client_exists(client_id.clone()) {
            let _ = ClientConfig::create_client_log_in_json(client_id.clone());
        }
        let file = std::fs::File::open(path.clone())?;
        let mut client_config: ClientConfig = serde_json::from_reader(file)?;
        let json = serde_json::to_string(&message)?;
        client_config.subscriptions.push(json);
        let json = serde_json::to_string(&client_config)?;
        std::fs::write(path, json)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    impl ClientConfig {
        /// Obtiene un cliente del archivo json
        pub fn get_client(client_id: String) -> ClientConfig {
            // obtiene un cliente del archivo json
            let path = format!("./src/mqtt/clients/{}.json", client_id);
            let file = std::fs::File::open(path).unwrap();
            serde_json::from_reader(file).unwrap()
        }
    }

    #[test]
    fn test_new_client_config() {
        let client_id = "test".to_string();
        let client_config = ClientConfig::new(client_id.clone());
        assert_eq!(client_config.client_id, client_id);
        assert!(client_config.state);
        assert_eq!(client_config.subscriptions.len(), 0);
    }

    #[test]
    fn test_change_client_state() {
        let client_id = "test".to_string();
        let _ = ClientConfig::create_client_log_in_json(client_id.clone());
        let _ = ClientConfig::change_client_state(client_id.clone(), false);
        let client_config = ClientConfig::get_client(client_id.clone());
        assert!(!client_config.state);
        ClientConfig::delete_client_file(client_id.clone()).unwrap();
    }

    #[test]
    fn test_create_client_log_in_json() {
        let client_id = "test".to_string();
        let _ = ClientConfig::create_client_log_in_json(client_id.clone());
        let path = format!("./src/mqtt/clients/{}.json", client_id);
        assert!(std::fs::metadata(path).is_ok());
        ClientConfig::delete_client_file(client_id.clone()).unwrap();
    }

    #[test]
    fn test_add_new_subscription() {
        let client_id = "test".to_string();
        let topic = "test".to_string();
        let _ = ClientConfig::create_client_log_in_json(client_id.clone());
        let _ = ClientConfig::add_new_subscription(client_id.clone(), topic.clone());
        let client_config = ClientConfig::get_client(client_id.clone());
        assert_eq!(client_config.subscriptions[0], topic);
        ClientConfig::delete_client_file(client_id.clone()).unwrap();
    }

    #[test]
    fn test_remove_subscription() {
        let client_id = "test".to_string();
        let topic = "test".to_string();
        let _ = ClientConfig::create_client_log_in_json(client_id.clone());
        let _ = ClientConfig::add_new_subscription(client_id.clone(), topic.clone());
        let _ = ClientConfig::remove_subscription(client_id.clone(), topic.clone());
        let client_config = ClientConfig::get_client(client_id.clone());
        assert_eq!(client_config.subscriptions.len(), 0);
        ClientConfig::delete_client_file(client_id.clone()).unwrap();
    }
}