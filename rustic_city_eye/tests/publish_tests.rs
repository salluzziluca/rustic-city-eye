use std::io::Cursor;

use rustic_city_eye::mqtt::{
    client_message, publish_properties::PublishProperties
};

#[test]
fn test_publish_message_ok() {
    let properties = PublishProperties::new(1, 10, 10, "String".to_string(), [1, 2, 3].to_vec(), "a".to_string(), 1, "a".to_string());

    let publish = client_message::ClientMessage::Publish { packet_id: 1, topic_name: "mensajes para juan".to_string(), qos: 1, retain_flag: true, payload: "hola soy juan".to_string(), dup_flag: true, properties };
    
    let mut cursor = Cursor::new(Vec::<u8>::new());
    publish.write_to(&mut cursor).unwrap();
    cursor.set_position(0);


    match client_message::ClientMessage::read_from(&mut cursor) {
        Ok(read_publish) => {
            assert_eq!(publish, read_publish);
        }
        Err(e) => {
            panic!("no se pudo leer del cursor {:?}", e);
        }
    }
}