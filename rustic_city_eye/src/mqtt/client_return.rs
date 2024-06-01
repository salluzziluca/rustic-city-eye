#[derive(Debug, PartialEq)]
pub enum ClientReturn {
    PubackRecieved,
    SubackSent,
    PubackSent,
    PingrespSent,
    UnsubackSent,
    DisconnectRecieved,
    PlaceHolder,
    DisconnectSent,
}
