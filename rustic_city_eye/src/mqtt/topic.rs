use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use std::collections::HashMap;

use super::subscription::Subscription;

use super::reason_code;

/// Estructura que representa un topic.
#[derive(Debug, Clone)]
pub struct Topic {
    /// Hashmap de subscriptores.
    users: Arc<RwLock<Vec<Subscription>>>,
    // vector de subtopics
    subtopics: Vec<String>,
}

impl Default for Topic {
    fn default() -> Self {
        Self::new()
    }
}

impl Topic {
    /// Crea un nuevo topic sin subscriptores
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(Vec::new())),
            subtopics: Vec::new(),
        }
    }

    /// Agrega un usuario a un topic
    /// Si el usuario no existe en el tópic, lo agrega
    /// Si el usuario ya existe en el tópic, no lo agrega pero la operacion es exitosa de todas maneras ya que el objetivo era concretar la subscription
    /// Retorna el reason code de la operación
    pub fn add_user_to_topic(&mut self, subscription: Subscription) -> u8 {
        println!("Adding user to topic: {:?}", subscription);
        //verificar si el user_id ya existe en topic users
        let mut existe = false;
        match self.users.read() {
            Ok(guard) => {
                if guard.contains(&subscription) {
                    existe = true;
                }
            }
            Err(_) => return reason_code::UNSPECIFIED_ERROR_HEX,
        }

        let mut lock = match self.users.write() {
            Ok(guard) => guard,
            Err(_) => return reason_code::UNSPECIFIED_ERROR_HEX,
        };

        if !existe {
            lock.push(subscription);
        }

        reason_code::SUCCESS_HEX
    }

    /// Elimina un usuario de un topic
    /// Si el usuario no existe en el topic, no hace nada
    /// Retorna el reason code de la operación
    pub fn remove_user_from_topic(&mut self, subscription: Subscription) -> u8 {
        let mut lock = match self.users.write() {
            Ok(guard) => guard,
            Err(_) => return reason_code::UNSPECIFIED_ERROR_HEX,
        };

        if lock.contains(&subscription) {
            lock.retain(|x| x != &subscription);
        }

        reason_code::SUCCESS_HEX
    }

    pub fn add_subtopic(&mut self, topic: String) {
        self.subtopics.push(topic);
    }

    pub fn remove_subtopic(&mut self, topic: String) {
        self.subtopics.retain(|x| x != &topic);
    }

    pub fn get_subtopics(&self) -> Vec<String> {
        self.subtopics.clone()
    }

    pub fn get_users_from_topic(&self) -> Vec<Subscription> {
        let lock = match self.users.read() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };

        let mut users = Vec::new();
        for user in lock.iter() {
            users.push(user.clone());
        }

        users
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_new_topic() {
        let topic = Topic::new();
        assert_eq!(topic.users.read().unwrap().len(), 0);
    }

    #[test]
    fn test_add_subscriber() {
        let mut topic = Topic::new();
        let user_id = "user".to_string();
        let subscription = Subscription::new("topic".to_string(), user_id);
        let result = topic.add_user_to_topic(subscription);
        assert_eq!(result, 0x00);
    }

    #[test]
    fn test_add_subscriber_duplicate() {
        let mut topic = Topic::new();
        let user_id = "user".to_string();
        let subscription = Subscription::new("topic".to_string(), user_id);
        let result = topic.add_user_to_topic(subscription.clone());
        assert_eq!(result, 0x00);
        let result = topic.add_user_to_topic(subscription);
        assert_eq!(result, 0x00);
    }

    #[test]
    fn test_remove_subscriber() {
        let mut topic = Topic::new();
        let user_id = "user".to_string();
        let subscription = Subscription::new("topic".to_string(), user_id);
        let result = topic.add_user_to_topic(subscription.clone());
        assert_eq!(result, 0x00);
        let result = topic.remove_user_from_topic(subscription);
        assert_eq!(result, 0x00);
    }

    #[test]
    fn test_remove_subscriber_no_matching() {
        let mut topic = Topic::new();
        let user_id = "user".to_string();
        let subscription = Subscription::new("topic".to_string(), user_id);
        let result = topic.add_user_to_topic(subscription.clone());
        assert_eq!(result, 0x00);
        let result = topic.remove_user_from_topic(subscription);
        assert_eq!(result, 0x00);
    }

    #[test]
    fn test_add_subtopic() {
        let mut topic = Topic::new();
        let subtopic = "subtopic".to_string();
        topic.add_subtopic(subtopic.clone());
        let subtopic2 = "subtopic2".to_string();
        topic.add_subtopic(subtopic2.clone());
        assert_eq!(topic.get_subtopics().len(), 2);
        assert_eq!(topic.get_subtopics()[0], subtopic);
        assert_eq!(topic.get_subtopics()[1], subtopic2);
    }
}
