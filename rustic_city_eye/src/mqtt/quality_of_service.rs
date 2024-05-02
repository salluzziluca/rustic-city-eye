#[derive(Debug)]
pub enum QualityOfService {
    AtMostOnce,  //QoS = 0
    AtLeastOnce, //QoS = 1
    ExactlyOnce, //QoS = 2
}
