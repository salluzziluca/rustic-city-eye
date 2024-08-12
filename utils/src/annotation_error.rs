use std::fmt;

#[derive(Debug, PartialEq)]
pub enum AnnotationError {
    ApiKeyNotSet(String),
    ImageError(String),
    RequestError(String),
    KeywordsFileError(String),
}

impl fmt::Display for AnnotationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AnnotationError::ApiKeyNotSet(ref err) => {
                write!(f, "Error al leer la variable de entorno: {}", err)
            }
            AnnotationError::ImageError(ref err) => {
                write!(f, "Error al abrir la imagen a clasificar: {}", err)
            }
            AnnotationError::RequestError(ref err) => {
                write!(f, "Error al clasificar la imagen: {}", err)
            }
            AnnotationError::KeywordsFileError(ref err) => {
                write!(
                    f,
                    "Error al leer las palabras claves de incidentes: {}",
                    err
                )
            }
        }
    }
}