use std::{io::{Read, Write}, net::TcpStream, sync::{Arc, Mutex}};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Topic {
    subscribers: Arc<Mutex<Vec<TcpStream>>>
}

impl Topic {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(Vec::new()))
        }
    }

    // pub fn add_subscriber<T: Read + Write + Debug>(&mut self, subscriber_stream: T) {
    //     let mut subs = self.subscribers.lock().unwrap();
    //     subs.push(subscriber_stream);
    // }

    // pub fn send_message(self, message: &str) -> Result<(), std::io::Error> {
    //     for sub in self.subscribers {
    //         sub.send(message.to_string()).unwrap();
    //     }

    //     Ok(())
    // }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn test_01_topic_creation_ok() -> std::io::Result<()> {
        let _ = Topic::new();

        Ok(())
    }

    // #[test]
    // fn test_02_adding_sub_ok() -> std::io::Result<()> {
    //     let mut topic = Topic::new();

    //     let (tx, _rx) = mpsc::channel();
    //     topic.add_subscriber(tx);

    //     Ok(())
    // }

    // #[test]
    // fn test_03_adding_sub_and_sending_message_ok() {
    //     let mut topic = Topic::new();
    //     let (tx, rx) = mpsc::channel();
    //     topic.add_subscriber(tx);

    //     let _ = topic.send_message("holaholahola");

    //     let message_received = rx.recv().unwrap();

    //     assert_eq!(message_received, "holaholahola".to_string());
    // }

    // #[test]
    // fn test_04_adding_multiple_subs_and_sending_message_ok() {
    //     let mut topic = Topic::new();
    //     let (tx1, rx1) = mpsc::channel();
    //     let (tx2, rx2) = mpsc::channel();
    //     let (tx3, rx3) = mpsc::channel();

    //     topic.add_subscriber(tx1);
    //     topic.add_subscriber(tx2);
    //     topic.add_subscriber(tx3);

    //     let _ = topic.send_message("holaholahola");

    //     let message_received1 = rx1.recv().unwrap();
    //     let message_received2 = rx2.recv().unwrap();
    //     let message_received3 = rx3.recv().unwrap();

    //     assert_eq!(message_received1, "holaholahola".to_string());
    //     assert_eq!(message_received2, "holaholahola".to_string());
    //     assert_eq!(message_received3, "holaholahola".to_string());
    // }
}
