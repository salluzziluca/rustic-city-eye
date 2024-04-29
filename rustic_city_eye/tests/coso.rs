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
        last_will_message: "me he muerto, diganle a mi vieja que la quiero, adios".to_string(),
    };
    match connect {
        client_message::ClientMessage::Connect { clean_start, .. } => {
            assert_eq!(clean_start, true);
        }
        _ => panic!("Unexpected variant"),
    }
}
