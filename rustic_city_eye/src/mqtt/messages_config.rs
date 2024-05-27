// use crate::mqtt::{
//     publish_properties::PublishProperties,
//     connect_properties::ConnectProperties,
//     will_properties::WillProperties,
//     subscribe_properties::SubscribeProperties
// };

// pub enum MessagesConfig {
//     PublishConfig{
//         dup_flag: bool,
//         qos: usize,
//         retain_flag: bool,
//         topic_name: String,
//         payload: String,
//         publish_properties: PublishProperties
//     },
//     SubscribeConfig{
//         topic_name: String,
//         properties: SubscribeProperties,
//     },
//     UnsubscribeConfig{
//         topic_name: String,
//         properties: SubscribeProperties,
//     },
//     ConnectConfig{
//         clean_start: bool,
//         last_will_flag: bool,
//         last_will_qos: u8,
//         last_will_retain: bool,
//         keep_alive: u16,
//         properties: ConnectProperties,
//         client_id: String,
//         will_properties: WillProperties,
//         last_will_topic: String,
//         last_will_message: String,
//         username: String,
//         password: String,
//     }
// }

use crate::mqtt::client_message::ClientMessage;

pub trait MessagesConfig {
    fn parse_message(&self, packet_id: u16) -> ClientMessage;
}
