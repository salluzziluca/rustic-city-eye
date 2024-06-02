use std::fmt;

///Here are detailed all the errors that the protocol is capable of throwing.
/// Unspecified se usa de placeholder para los results de los tests
#[derive(Debug)]
pub enum ProtocolError {
    ConectionError,
    InvalidQOS,
    InvalidNumberOfArguments,
    StreamError,
    ReadingTopicConfigFileError,
    LockError,
    PublishError,
    SubscribeError,
    UnsubscribeError,
    UnspecifiedError,
    PubackWithoutPendingID,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProtocolError::ConectionError => write!(f, "Error al conectar al broker."),
            ProtocolError::InvalidQOS => write!(f, "Error: Valor de QoS inválido. Debe ser 0 o 1."),
            ProtocolError::InvalidNumberOfArguments => {
                write!(f, "Error: número de argumentos inválido")
            }
            ProtocolError::StreamError => write!(f, "Error: Error en la creación de un stream."),
            ProtocolError::LockError => write!(f, "Error en Lock."),
            ProtocolError::PublishError => write!(f, "Error al realizar un publish."),
            ProtocolError::SubscribeError => write!(f, "Error al realizar la subscripcion."),
            ProtocolError::UnsubscribeError => write!(f, "Error al desuscribirse."),

            ProtocolError::ReadingTopicConfigFileError => {
                write!(f, "Error al leer el archivo de configuración de tópicos")
            }
            ProtocolError::PubackWithoutPendingID => {
                write!(
                    f,
                    "Error: Se recibió un Puback sin un ID en la lista de pending message"
                )
            }
            ProtocolError::UnspecifiedError => {
                write!(f, "Error no especificado")
            }
        }
    }
}
