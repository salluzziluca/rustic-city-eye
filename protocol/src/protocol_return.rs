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
    AuthRecieved,
    NoAckSent,
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
            ProtocolReturn::AuthRecieved => write!(f, "Auth Recibido."),
            ProtocolReturn::NoAckSent => write!(f, "No Ack Enviado."),
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_display_protocol_return() {
        assert_eq!(ProtocolReturn::ConnackSent.to_string(), "Connack Enviado.");
        assert_eq!(ProtocolReturn::SubackSent.to_string(), "Suback Enviado.");
        assert_eq!(ProtocolReturn::PubackSent.to_string(), "Puback Enviado.");
        assert_eq!(
            ProtocolReturn::PingrespSent.to_string(),
            "Pingresp Enviado."
        );
        assert_eq!(
            ProtocolReturn::UnsubackSent.to_string(),
            "Unsuback Enviado."
        );
        assert_eq!(
            ProtocolReturn::DisconnectRecieved.to_string(),
            "Disconnect Recibido."
        );
        assert_eq!(ProtocolReturn::PlaceHolder.to_string(), "Placeholder.");
        assert_eq!(
            ProtocolReturn::DisconnectSent.to_string(),
            "Disconnect Enviado."
        );
        assert_eq!(ProtocolReturn::AuthRecieved.to_string(), "Auth Recibido.");
    }
}