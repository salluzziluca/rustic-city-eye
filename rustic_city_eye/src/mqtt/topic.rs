use std::{net::TcpStream, sync::{Arc, Mutex}};

#[derive(Debug)]
pub struct Topic{
    topic_name: String,
    subscribers: Arc<Mutex<Vec<TcpStream>>>
}

impl Topic {
    pub fn new(topic_name: &str) -> Self {
        Self { 
            topic_name: topic_name.to_string(),
            subscribers: Arc::new(Mutex::new(Vec::new()))
        }
    }

    pub fn add_subscriber(&mut self, subscriber_stream: TcpStream) {
        let subscribers = self.subscribers.lock().unwrap();
    }   
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_topic_creation_ok() -> std::io::Result<()> {
        let _ = Topic::new("mensajes-para-juan");

        Ok(())
    }

    #[test]
    fn test_02_adding_sub_ok() -> std::io::Result<()> {
        let topic = Topic::new("mensajes-para-juan");
        


        Ok(())
    }
}