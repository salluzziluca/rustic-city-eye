use std::io::Cursor;

use rustic_city_eye::mqtt::{client_message, connack_properties::ConnackPropertiesBuilder};

#[test]
fn test_connack() {
    let properties = ConnackPropertiesBuilder::new()
        .session_expiry_interval(100)
        .receive_maximum(10)
        .maximum_qos(true)
        .retain_available(true)
        .maximum_packet_size(100)
        .assigned_client_identifier("client_id".to_owned())
        .topic_alias_maximum(10)
        .reason_string("reason".to_owned())
        .user_properties(vec![("property".to_string(), "value".to_string())])
        .wildcard_subscription_available(true)
        .subscription_identifier_available(true)
        .shared_subscription_available(true)
        .server_keep_alive(10)
        .response_information("Testing".to_owned())
        .server_reference("server".to_owned())
        .authentication_method("auth".to_owned())
        .authentication_data("data".to_owned().as_bytes().to_vec()) // Convert string to Vec<u8>
        .build()
        .unwrap();

    println!("{:?}", properties);
    let connack = client_message::ClientMessage::Connack {
        session_present: true,
        reason_code: 0,
        properties: properties,
    };

    let mut cursor = Cursor::new(Vec::<u8>::new());
    connack.write_to(&mut cursor).unwrap();
    cursor.set_position(0);

    match client_message::ClientMessage::read_from(&mut cursor) {
        Ok(read_connack) => {
            assert_eq!(connack, read_connack);
        }
        Err(e) => {
            panic!("Failed to read from cursor: {:?}", e);
        }
    }
}
