use std::io::Cursor;

use rustic_city_eye::mqtt::client_message;

#[test]
fn test_client_message() {
    let connect = client_message::ClientMessage::Connect {
        clean_start: true,
        last_will_flag: true,
        last_will_qos: 1,
        last_will_retain: true,
        username: "prueba".to_string(),
        password: "".to_string(),
        keep_alive: 35,
        client_id: "kvtr33".to_string(),
        last_will_delay_interval: 15,
        message_expiry_interval: 120,
        content_type: "plain".to_string(),
        user_property: Some(("propiedad".to_string(), "valor".to_string())),
        last_will_message: "chauchis".to_string(),
        response_topic: "algo".to_string(),
        correlation_data: vec![1, 2, 3, 4, 5],
        payload_format_indicator: 1,
        lastWillTopic: "topico".to_string(),
    };
    let mut cursor = Cursor::new(Vec::<u8>::new());
    connect.write_to(&mut cursor).unwrap();
    cursor.set_position(0);

    let read_connect = client_message::ClientMessage::read_from(&mut cursor).unwrap();

    assert_eq!(connect, read_connect);
}
