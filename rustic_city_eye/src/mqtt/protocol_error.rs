use std::fmt;

///Here are detailed all the errors that the protocol is capable of throwing.
/// Unspecified se usa de placeholder para los results de los tests
#[derive(Debug, PartialEq)]
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
    UnspecifiedError(String),
    PubackWithoutPendingID,
    WriteError,
    ReadingConfigFileError,
    MissingWillMessageProperties,
    ChanellError(String),
    ReadingClientsFileError,
    NotReceivedMessageError,
    ExpectedConnack,
    AuthError,
    AbnormalDisconnection,
    DroneError(String),
    CameraError(String),
    SendError(String),
    ShutdownError(String),
    BindingError(String),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProtocolError::ConectionError => write!(f, "Error al conectar al broker."),
            ProtocolError::ReadingConfigFileError => {
                write!(f, "Error al leer el archivo de configuracion del connect.")
            }
            ProtocolError::NotReceivedMessageError => {
                write!(f, "Error: no ha llegado ningun mensaje.")
            }
            ProtocolError::AuthError => {
                write!(f, "Error al autenticar al cliente")
            }
            ProtocolError::ExpectedConnack => {
                write!(f, "Error: no ha llegado un mensaje del tipo connack.")
            }
            ProtocolError::ReadingClientsFileError => {
                write!(f, "Error al leer el archivo de clientes.")
            }
            ProtocolError::MissingWillMessageProperties => {
                write!(f, "Error: faltan propiedades del will message.")
            }
            ProtocolError::WriteError => write!(f, "Error escribir el mensaje."),
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
            ProtocolError::UnspecifiedError(ref err) => {
                write!(f, "Error no especificado: {}", err)
            }
            ProtocolError::ChanellError(ref err) => {
                write!(
                    f,
                    "Error al enviar o recibir mensajes por el canal: {}",
                    err
                )
            }
            ProtocolError::AbnormalDisconnection => {
                write!(f, "Error: desconexión anormal.")
            }
            ProtocolError::DroneError(ref err) => {
                write!(f, "Error de protocolo: {}", err)
            }
            ProtocolError::ShutdownError(ref err) => {
                write!(f, "Error de protocolo: {}", err)
            }
            ProtocolError::BindingError(ref err) => {
                write!(f, "Error de protocolo: {}", err)
            }
            ProtocolError::CameraError(ref err) => {
                write!(f, "Error de protocolo: {}", err)
            }
            ProtocolError::SendError(ref err) => {
                write!(f, "Error al enviar mensaje: {}", err)
            }
        }
    }
}

#[cfg(test)]
#[test]
fn test_display_protocol_error() {
    assert_eq!(
        ProtocolError::ConectionError.to_string(),
        "Error al conectar al broker."
    );
    assert_eq!(
        ProtocolError::InvalidQOS.to_string(),
        "Error: Valor de QoS inválido. Debe ser 0 o 1."
    );
    assert_eq!(
        ProtocolError::InvalidNumberOfArguments.to_string(),
        "Error: número de argumentos inválido"
    );
    assert_eq!(
        ProtocolError::StreamError.to_string(),
        "Error: Error en la creación de un stream."
    );
    assert_eq!(ProtocolError::LockError.to_string(), "Error en Lock.");
    assert_eq!(
        ProtocolError::PublishError.to_string(),
        "Error al realizar un publish."
    );
    assert_eq!(
        ProtocolError::SubscribeError.to_string(),
        "Error al realizar la subscripcion."
    );
    assert_eq!(
        ProtocolError::UnsubscribeError.to_string(),
        "Error al desuscribirse."
    );
    assert_eq!(
        ProtocolError::ReadingTopicConfigFileError.to_string(),
        "Error al leer el archivo de configuración de tópicos"
    );
    assert_eq!(
        ProtocolError::PubackWithoutPendingID.to_string(),
        "Error: Se recibió un Puback sin un ID en la lista de pending message"
    );
    assert_eq!(
        ProtocolError::UnspecifiedError("Error".to_string()).to_string(),
        "Error no especificado: Error"
    );
}
