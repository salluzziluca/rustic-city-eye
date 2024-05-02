use std::{io::Cursor, ptr::null};

use rustic_city_eye::mqtt::{
    client_message,
    will_properties::{self, WillProperties},
};

#[test]
fn test_client_message() {
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
fn test_sin_will_prop() {
    let connect = client_message::ClientMessage::Connect {
        clean_start: true,
        last_will_flag: true,
        last_will_qos: 1,
        last_will_retain: true,
        keep_alive: 35,
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
