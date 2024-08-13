use super::will_properties::WillProperties;

pub struct LastWill {
    topic: String,
    message: String,
    qos: u8,
    retain: bool,
    properties: WillProperties,
}

impl LastWill {
    pub fn new(
        topic: String,
        message: String,
        qos: u8,
        retain: bool,
        properties: WillProperties,
    ) -> LastWill {
        LastWill {
            topic,
            message,
            qos,
            retain,
            properties,
        }
    }

    pub fn get_topic(&self) -> &String {
        &self.topic
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }

    pub fn get_qos(&self) -> u8 {
        self.qos
    }

    pub fn get_retain(&self) -> bool {
        self.retain
    }

    pub fn get_properties(&self) -> &WillProperties {
        &self.properties
    }
}
