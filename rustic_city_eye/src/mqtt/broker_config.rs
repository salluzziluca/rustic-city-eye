use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

use crate::mqtt::{protocol_error::ProtocolError, topic::Topic};

/// Contiene la configuracion del Broker:
/// Lee el archivo de topics que el usuario quiera manejar, y
/// guarda el address que usa.
pub struct BrokerConfig {
    address: String,
    topics: HashMap<String, Topic>,
}

impl BrokerConfig {
    ///Leo el archivo de topic_config y guardo las lecturas que haga.
    pub fn new(address: String) -> Result<BrokerConfig, ProtocolError> {
        let mut topics = HashMap::new();
        let topic_config_path = "./src/monitoring/topics.txt";

        let readings = BrokerConfig::process_topic_config_file(topic_config_path)?;

        for topic in readings {
            topics.insert(topic, Topic::new());
        }

        Ok(BrokerConfig { address, topics })
    }

    /// La idea es que Broker llame a esta funcion y que pueda acceder a su configuracion
    pub fn get_broker_config(&self) -> (String, HashMap<String, Topic>) {
        (self.address.clone(), self.topics.clone())
    }

    fn process_topic_config_file(file_path: &str) -> Result<Vec<String>, ProtocolError> {
        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(_) => return Err(ProtocolError::ReadingTopicConfigFileError),
        };

        let readings = BrokerConfig::read_topic_config_file(&file)?;

        Ok(readings)
    }

    fn read_topic_config_file(file: &File) -> Result<Vec<String>, ProtocolError> {
        let reader = BufReader::new(file).lines();
        let mut readings = Vec::new();

        for line in reader {
            match line {
                Ok(line) => readings.push(line),
                Err(_err) => return Err(ProtocolError::ReadingTopicConfigFileError),
            }
        }

        Ok(readings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_broker_config() {
        let address = "127.0.0.1::5000".to_string();
        let broker_config = BrokerConfig::new(address.clone()).unwrap();
        let (broker_address, topics) = broker_config.get_broker_config();
        assert!(broker_address == address);
        assert!(topics.len() == 5);
        assert!(topics.contains_key("incidente"));
        assert!(topics.contains_key("fulbito"));
        assert!(topics.contains_key("mensajes para juan"));
        assert!(topics.contains_key("accidente"));
        assert!(topics.contains_key("messi"));
    }

    #[test]
    fn test_process_topic_config_file() {
        let topic_config_path = "./src/monitoring/topics_test.txt";
        let readings = BrokerConfig::process_topic_config_file(topic_config_path).unwrap();

        assert!(readings.len() == 4);
        assert!(readings.contains(&"Luca".to_string()));
        assert!(readings.contains(&"Juan".to_string()));
        assert!(readings.contains(&"Lihuen".to_string()));
        assert!(readings.contains(&"Nacho".to_string()));
    }
}
