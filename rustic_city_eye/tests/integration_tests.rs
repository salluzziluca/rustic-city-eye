use mockall::{automock, predicate::*};
use rustic_city_eye::mqtt::client_message::ClientMessage;

// Define a trait that represents the behavior of the Client that you want to mock
#[automock]
pub trait ClientTrait {
    fn subscribe(&mut self, topic: &str) -> Result<(), String>;
    fn publish(&mut self, topic: &str, message: &str) -> Result<(), String>;
}

use std::io::Cursor;
use std::io::Read;

// In your tests, you can now create a MockClient and set up expectations
#[cfg(test)]
mod tests {
    use std::io::Bytes;

    use super::*;
    use rustic_city_eye::mqtt::subscribe_properties::SubscribeProperties;

    #[test]
    fn test_broker() {
        let mut mock_client = MockClientTrait::new();

        // Set up expectations
        mock_client
            .expect_subscribe()
            .with(eq("accidente"))
            .returning(|_| Ok(()));

        // Create a Cursor over your messages
        let subscribe = ClientMessage::Subscribe {
            packet_id: 1,
            topic_name: "topic".to_string(),
            properties: SubscribeProperties::new(
                1,
                vec![("propiedad".to_string(), "valor".to_string())],
                vec![0, 1, 2, 3],
            ),
        };
        let mut cursor = Cursor::new();

        // Read from the Cursor as if it were a client sending messages
        let mut buffer = Vec::new();
        cursor.read_to_end(&mut buffer).unwrap();

        // Use the mock in your test
        let mut broker = Broker::new(vec!["app".to_string(), "8080".to_string()]).unwrap();
        let result = broker.handle_client(buffer);
        assert!(result.is_ok());
    }
}
