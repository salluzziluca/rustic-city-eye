use serde::{Deserialize, Serialize};

/// Subscription struct que se usa para almacenar la suscripción de un cliente a un topic
#[derive(Debug, Eq, Hash, PartialEq, Serialize, Deserialize, Clone)]
pub struct Subscription {
    /// Topic al que se quiere suscribir
    pub topic: String,
    /// Id del cliente que se suscribe
    pub client_id: String,
}

/// Implementación de Subscription
impl Subscription {
    pub fn new(topic: String, client_id: String) -> Subscription {
        Subscription { topic, client_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_subscription() {
        let topic = "topic".to_string();
        let client_id = "client".to_string();
        let subscription = Subscription::new(topic.clone(), client_id.clone());
        assert_eq!(subscription.topic, topic);
    }
}
