use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Error;
use std::net::TcpStream;
use std::sync::{Arc, RwLock};

use crate::mqtt::broker_message::BrokerMessage;

use super::reason_code;


#[derive(Debug, Clone)]
pub struct Topic {
    /// Hashmap de subscriptores.
    /// El u32 representa el sub_id, y el valor es el stream del subscriptor.
    subscribers: Arc<RwLock<HashMap<u32, TcpStream>>>,
}

impl Default for Topic {
    fn default() -> Self {
        Self::new()
    }
}

impl Topic {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_subscriber(&mut self, stream: TcpStream, sub_id: u32) -> u8 {
        //verificar si el sub_id ya existe en topic subscribers
        if self.subscribers.read().unwrap().contains_key(&sub_id) {
            return reason_code::SUB_ID_DUP_HEX;
        }

        let mut lock = match self.subscribers.write() {
            Ok(guard) => guard,
            Err(_) => return reason_code::UNSPECIFIED_ERROR_HEX,
        };

        lock.insert(sub_id, stream);
        reason_code::SUCCESS_HEX
    }

    // pub fn remove_subscriber(&mut self, stream: TcpStream) -> Result<(), ProtocolError> {
    //     let mut lock = match self.subscribers.write() {
    //         Ok(guard) => guard,
    //         Err(_) => return Err(ProtocolError::LockError)
    //     };

    //     //let sub_index = lock.iter().position(|&r| r == stream).unwrap();
    //     //println!("Encontre el sub en la pos {}", sub_index);

    //     Ok(())
    // }

    pub fn deliver_message(&self, payload: String) -> Result<u8, Error> {
        let lock = self.subscribers.read().unwrap();
        let mut puback_reason_code: u8 = 0x00;

        let delivery_message = BrokerMessage::PublishDelivery { payload };

        if lock.is_empty() {
            puback_reason_code = 0x10_u8;
            return Ok(puback_reason_code);
        }

        for mut subscriber in lock.iter() {
            println!("Enviando un PublishDelivery");
            match delivery_message.write_to(&mut subscriber.1) {
                Ok(_) => println!("PublishDelivery enviado"),
                Err(err) => println!("Error al enviar PublishDelivery: {:?}", err),
            }
        }

        Ok(puback_reason_code)
    }
}

#[cfg(test)]
mod tests {
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
