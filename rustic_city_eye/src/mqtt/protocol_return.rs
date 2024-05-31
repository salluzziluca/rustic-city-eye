use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ProtocolReturn {
    ConnackSent,
    SubackSent,
    PubackSent,
    PingrespSent,
    UnsubackSent,
    DisconnectRecieved,
    PlaceHolder,
    DisconnectSent,
}

impl fmt::Display for ProtocolReturn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProtocolReturn::ConnackSent => write!(f, "Connack Enviado."),
            ProtocolReturn::SubackSent => write!(f, "Suback Enviado."),
            ProtocolReturn::PubackSent => write!(f, "Puback Enviado."),
            ProtocolReturn::PingrespSent => write!(f, "Pingresp Enviado."),
            ProtocolReturn::UnsubackSent => write!(f, "Unsuback Enviado."),
            ProtocolReturn::DisconnectRecieved => write!(f, "Disconnect Recibido."),
            ProtocolReturn::PlaceHolder => write!(f, "Placeholder."),
            ProtocolReturn::DisconnectSent => write!(f, "Disconnect Enviado."),
        }
    }
}
