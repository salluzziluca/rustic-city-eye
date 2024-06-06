
// UserSubscription es una estructura que representa una suscripción de un usuario a un topic
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct UserSubscription {
    pub user_id: u8,
    pub qos: u8,
    // no_local,
    // retain_as_published,
    // retain_handling,
}

impl UserSubscription {
    pub fn new(user_id: u8, qos: u8) -> UserSubscription {
        UserSubscription { user_id, qos }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_subscription() {
        let user_id = 1;
        let qos = 0x01;
        let subscription = UserSubscription::new(user_id, qos);
        assert_eq!(subscription.user_id, user_id);
        assert_eq!(subscription.qos, qos);
    }
}