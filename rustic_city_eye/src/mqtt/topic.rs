use std::fmt::Debug;
use std::io::Error;
use std::sync::{Arc, RwLock};

use crate::mqtt::broker_message::BrokerMessage;

use super::client_message::ClientMessage;

use super::subscription::Subscription;

use super::reason_code;

/// Estructura que representa un tópic.
#[derive(Debug, Clone)]
pub struct Topic {
    /// Hashmap de subscriptores.
    users: Arc<RwLock<Vec<Subscription>>>,
    // vector de hijos
    // hijos: Vec<Topic>,
}

impl Default for Topic {
    fn default() -> Self {
        Self::new()
    }
}

impl Topic {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(Vec::new())),
        }
    }

    // Agrega un usuario a un tópic
    // Si el usuario ya existe en el tópic, lo elimina y lo vuelve a agregar
    // Si el usuario no existe en el tópic, lo agrega
    // Retorna el reason code de la operación
    pub fn add_user_to_topic(&mut self, subscription: Subscription) -> u8 {
        //verificar si el user_id ya existe en topic users
        let mut existe = false;
        match self.users.read() {
            Ok(guard) => {
                if guard.contains(&subscription) {
                    existe = true;
                }
            }
            Err(_) => return reason_code::UNSPECIFIED_ERROR_HEX,
        }

        let mut lock = match self.users.write() {
            Ok(guard) => guard,
            Err(_) => return reason_code::UNSPECIFIED_ERROR_HEX,
        };

        if !existe {
            lock.push(subscription);
        }

        reason_code::SUCCESS_HEX
    }

    pub fn remove_user_from_topic(&mut self, subscription: Subscription) -> u8 {
        let mut lock = match self.users.write() {
            Ok(guard) => guard,
            Err(_) => return reason_code::UNSPECIFIED_ERROR_HEX,
        };

        if lock.contains(&subscription) {
            lock.retain(|x| x != &subscription);
        }

        reason_code::SUCCESS_HEX
    }

    pub fn deliver_message(&self, message: ClientMessage) -> Result<u8, Error> {
        let lock = self.users.read().unwrap();
        let mut puback_reason_code: u8 = 0x00;
        match message {
            ClientMessage::Publish {
                topic_name,
                payload,
                packet_id,
                qos,
                retain_flag,
                dup_flag,
                properties,
            } => {
                let delivery_message = BrokerMessage::PublishDelivery {
                    topic_name,
                    payload,
                    packet_id,
                    qos,
                    retain_flag,
                    dup_flag,
                    properties,
                };

                if lock.is_empty() {
                    puback_reason_code = 0x10_u8;
                    return Ok(puback_reason_code);
                }

                // for mut subscriber in lock.iter() {
                //     println!("Enviando un PublishDelivery");
                //     match delivery_message.write_to(&mut subscriber) {
                //         Ok(_) => println!("PublishDelivery enviado"),
                //         Err(err) => println!("Error al enviar PublishDelivery: {:?}", err),
                //     }
                // }
            }
            _ => {
                return Ok(0x10);
            }
        };
        Ok(puback_reason_code)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_new_topic() {
        let topic = Topic::new();
        assert_eq!(topic.users.read().unwrap().len(), 0);
    }

    #[test]
    fn test_add_subscriber() {
        let mut topic = Topic::new();
        let user_id = "user".to_string();
        let qos = 1;
        let subscription = Subscription::new("topic".to_string(), user_id, qos);
        let result = topic.add_user_to_topic(subscription);
        assert_eq!(result, 0x00);
    }

    #[test]
    fn test_add_subscriber_duplicate() {
        let mut topic = Topic::new();
        let user_id = "user".to_string();
        let subscription = Subscription::new("topic".to_string(), user_id, 1);
        let result = topic.add_user_to_topic(subscription.clone());
        assert_eq!(result, 0x00);
        let result = topic.add_user_to_topic(subscription);
        assert_eq!(result, 0x00);
    }

    #[test]
    fn test_remove_subscriber() {
        let mut topic = Topic::new();
        let user_id = "user".to_string();
        let subscription = Subscription::new("topic".to_string(), user_id, 1);
        let result = topic.add_user_to_topic(subscription.clone());
        assert_eq!(result, 0x00);
        let result = topic.remove_user_from_topic(subscription);
        assert_eq!(result, 0x00);
    }

    #[test]
    fn test_remove_subscriber_no_matching() {
        let mut topic = Topic::new();
        let user_id = "user".to_string();
        let subscription = Subscription::new("topic".to_string(), user_id, 1);
        let result = topic.add_user_to_topic(subscription.clone());
        assert_eq!(result, 0x00);
        let result = topic.remove_user_from_topic(subscription);
        assert_eq!(result, 0x00);
    }
}
