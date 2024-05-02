use std::{io::Cursor, vec};

use rustic_city_eye::mqtt::will_properties::WillProperties;

#[test]
fn test_will_prop() {
    let will_properties = WillProperties::new(
        120,
        1,
        30,
        "plain".to_string(),
        "topic".to_string(),
        vec![1, 2, 3, 4, 5],
        vec![("propiedad".to_string(), "valor".to_string())],
    );
    let mut cursor = Cursor::new(Vec::<u8>::new());
    will_properties.write_to(&mut cursor).unwrap();
    cursor.set_position(0);

    let read_will_properties = WillProperties::read_from(&mut cursor).unwrap();

    assert_eq!(will_properties, read_will_properties);
}
