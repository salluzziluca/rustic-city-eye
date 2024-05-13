use mockall::{automock, predicate::*};
use rustic_city_eye::mqtt::client_message::ClientMessage;

// Define a trait that represents the behavior of the Client that you want to mock
#[cfg_attr(test, automock)]
pub trait ClientMessageTrait {
    fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()>;
}

use std::io::Cursor;
use std::io::Read;
use std::io::Write;

// In your tests, you can now create a MockClient and set up expectations
#[cfg(test)]
mod tests {
    use std::io::Bytes;

    use super::*;
    use rustic_city_eye::mqtt::{broker::Broker, subscribe_properties::SubscribeProperties};
    use std::borrow::Borrow;

    #[test]
    fn test_broker() {
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::<u8>::new());
        let mut mock_client: MockClientMessageTrait = MockClientMessageTrait::new();

        mock_client.expect_write_to().times(1).returning(|stream| {
            let _ = stream.write(&[0x82, 0x02, 0x00, 0x01]);
            Ok(())
        });
        // Read from the Cursor as if it were a client sending messages
        let mut buffer = Vec::new();
        cursor.read_to_end(&mut buffer).unwrap();

        // Use the mock in your test
        let mut broker = Broker::new(vec!["app".to_string(), "8080".to_string()]).unwrap();
        let result = broker.handle_client(&mut cursor);
        assert!(result.is_ok());
    }
}
