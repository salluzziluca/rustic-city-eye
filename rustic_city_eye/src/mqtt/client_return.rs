#[derive(Debug, PartialEq)]
pub enum ClientReturn {
    PubackRecieved,
    DisconnectRecieved,
    DisconnectSent,
    SubackRecieved,
    PublishDeliveryRecieved,
    UnsubackRecieved,
    PingrespRecieved,
    PlaceHolder,
}
