use std::io::Cursor;

use rustic_city_eye::mqtt::{client_message, connect_propierties, will_properties::WillProperties};

#[test]
fn test_client_message() {
    let connect_propierties = connect_propierties::ConnectProperties {
        session_expiry_interval: 1,
        receive_maximum: 2,
        maximum_packet_size: 10,
        topic_alias_maximum: 99,
        request_response_information: true,
        request_problem_information: false,
        user_properties: vec![
            ("Hola".to_string(), "Mundo".to_string()),
            ("Chau".to_string(), "Mundo".to_string()),
        ],
        authentication_method: "test".to_string(),
        authentication_data: vec![1_u8, 2_u8, 3_u8, 4_u8, 5_u8],
    };
    let will_properties = WillProperties::new(
        120,
        1,
        30,
        "plain".to_string(),
        "topic".to_string(),
        vec![1, 2, 3, 4, 5],
        vec![("propiedad".to_string(), "valor".to_string())],
    );
    let connect = client_message::ClientMessage::Connect {
        clean_start: true,
        last_will_flag: true,
        last_will_qos: 1,
        last_will_retain: true,
        keep_alive: 35,
        properties: connect_propierties,
        client_id: "kvtr33".to_string(),
        will_properties,
        last_will_topic: "topic".to_string(),
        last_will_message: "chauchis".to_string(),
        username: "prueba".to_string(),
        password: "".to_string(),
    };
    let mut cursor = Cursor::new(Vec::<u8>::new());
    connect.write_to(&mut cursor).unwrap();
    cursor.set_position(0);

    match client_message::ClientMessage::read_from(&mut cursor) {
        Ok(read_connect) => {
            assert_eq!(connect, read_connect);
        }
        Err(e) => {
            panic!("no se pudo leer del cursor {:?}", e);
        }
    }
}

#[test]
fn test_sin_props() {
    let connect = client_message::ClientMessage::Connect {
        clean_start: true,
        last_will_flag: true,
        last_will_qos: 1,
        last_will_retain: true,
        keep_alive: 35,
        properties: connect_propierties::ConnectProperties::new(
            0,
            0,
            0,
            0,
            false,
            false,
            vec![],
            "".to_string(),
            vec![],
        ),
        client_id: "kvtr33".to_string(),
        will_properties: WillProperties::new(
            0,
            1,
            0,
            "".to_string(),
            "".to_string(),
            vec![],
            vec![],
        ),
        last_will_topic: "topic".to_string(),
        last_will_message: "chauchis".to_string(),
        username: "prueba".to_string(),
        password: "".to_string(),
    };
    let mut cursor = Cursor::new(Vec::<u8>::new());
    connect.write_to(&mut cursor).unwrap();
    cursor.set_position(0);

    match client_message::ClientMessage::read_from(&mut cursor) {
        Ok(read_connect) => {
            assert_eq!(connect, read_connect);
        }
        Err(e) => {
            panic!("no se pudo leer del cursor {:?}", e);
        }
    }
}
