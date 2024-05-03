use std::io::Cursor;

use rustic_city_eye::mqtt::{client_message, subscribe_properties::SubscribeProperties};

#[test]
fn test_sub() {
    let sub = client_message::ClientMessage::Subscribe {
        packet_id: 1,
        topic_name: "topico".to_string(),
        properties: SubscribeProperties::new(
            1,
            vec![("propiedad".to_string(), "valor".to_string())],
            vec![0, 1, 2, 3],
        ),
    };

    let mut cursor = Cursor::new(Vec::<u8>::new());
    sub.write_to(&mut cursor).unwrap();
    cursor.set_position(0);
    let read_sub = client_message::ClientMessage::read_from(&mut cursor).unwrap();
    assert_eq!(sub, read_sub);
}
