#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct Subscription {
    pub topic: String,
    pub client_id: String,
}

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
