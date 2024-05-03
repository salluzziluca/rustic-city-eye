#[derive(Debug)]
pub enum QualityOfService {
    _AtMostOnce,  //QoS = 0
    _AtLeastOnce, //QoS = 1
}
