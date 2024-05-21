use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

pub struct User {
    pub user_id: String,    
    pub topics: HashMap<String, Topic>,
}

impl User {
    
    pub fn new(user_id: String) -> Result<User, ProtocolError> {
        let mut topics = HashMap::new();
        
        Ok(User { user_id, topics })
    }

    pub fn check_topic(&self, topic: &str) -> bool {
        self.topics.contains_key(topic)
    }

    pub fn get_client_config(&self) -> (String, HashMap<String, Topic>) {
        (self.user_id.clone(), self.topics.clone())
    }

    pub fn add_topic(&mut self, topic: String) {
        self.topics.insert(topic, Topic::new());
    }

    pub fn remove_topic(&mut self, topic: &str) {
        self.topics.remove(topic);
    }


}