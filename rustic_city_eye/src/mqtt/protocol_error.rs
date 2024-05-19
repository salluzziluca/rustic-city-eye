use std::fmt;

///Here are detailed all the errors that the protocol is capable of throwing.
#[derive(Debug)]
pub enum ProtocolError {
    ConectionError,
    InvalidQOS,
    InvalidNumberOfArguments,
    StreamError,
    ReadingTopicConfigFileError,
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
            ProtocolError::ReadingTopicConfigFileError => {
                write!(f, "Error al leer el archivo de configuración de tópicos")
            }
        }
    }
}
