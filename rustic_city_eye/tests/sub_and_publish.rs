// use rustic_city_eye::mqtt::{
//     broker::Broker, client_message, subscribe_properties::SubscribeProperties,
// };
// use std::io::Cursor;

// #[test]
// fn test_sub_y_pub() -> std::io::Result<()> {
//     let mut broker = Broker::new(vec![]).unwrap();
//     let mut subscribe_message = client_message::ClientMessage::Subscribe {
//         packet_id: 1,
//         topic_name: "topico".to_string(),
//         properties: SubscribeProperties::new(
//             1,
//             vec![("propiedad".to_string(), "valor".to_string())],
//             vec![0, 1, 2, 3],
//         ),
//     };

//     let mut stream1 = Cursor::new(Vec::new());
//     let mut stream2 = Cursor::new(Vec::new());

//     subscribe_message.write_to(&mut stream1).unwrap();

//     broker.handle_client(&mut stream1).unwrap();

//     Ok(())
// }
