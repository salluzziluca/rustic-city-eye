#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct Subscription {
    pub topic: String,
    pub client_id: String,
    pub qos: u8,
}

impl Subscription {
    pub fn new(topic: String, client_id: String, qos: u8) -> Subscription {
        Subscription { topic, client_id, qos }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_subscription() {
        let topic = "topic".to_string();
        let qos = 0x01;
        let client_id = "client".to_string();
        let subscription = Subscription::new(topic.clone(), client_id.clone(), qos);
        assert_eq!(subscription.topic, topic);
        assert_eq!(subscription.qos, qos);
    }
}
