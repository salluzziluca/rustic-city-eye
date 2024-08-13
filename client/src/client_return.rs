#[derive(Debug, PartialEq)]
pub enum ClientReturn {
    ConnackReceived,
    PubackRecieved,
    DisconnectRecieved,
    DisconnectSent,
    SubackRecieved,
    PublishDeliveryRecieved,
    UnsubackRecieved,
    PingrespRecieved,
    AuthRecieved,
    PlaceHolder,
}