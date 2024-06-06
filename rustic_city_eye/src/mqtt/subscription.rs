#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct Subscription {
    pub topic: String,
    pub qos: u8,
}

impl Subscription {
    pub fn new(topic: String, qos: u8) -> Subscription {
        Subscription { topic, qos }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_subscription() {
        let topic = "topic".to_string();
        let qos = 0x01;
        let subscription = Subscription::new(topic.clone(), qos);
        assert_eq!(subscription.topic, topic);
        assert_eq!(subscription.qos, qos);
    }
}
