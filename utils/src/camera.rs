use std::{
    fs,
    io::{Error, Read, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::{annotation::ImageClassifier, camera_error::CameraError, location::Location, protocol_error::ProtocolError, reader::*, writer::*};

const PATH: &str = "./camera_system/cameras";
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    /// Esta sera la localizacion que tendra la camara dentro del sistema.
    location: Location,

    /// Cada camara se identifica con un ID unico, el cual sera brindado por el sistema
    /// central de camaras.
    pub id: u32,

    /// Las cámaras inician su funcionamiento en modo ahorro de energia y pasan
    /// a estado de alerta cuando se recibe una notificación de un incidente en
    /// su area de alcance o en la de una cámara lindante.
    sleep_mode: bool,

    /// A traves de este clasificador se haran las peticiones al proveedor de la tecnologia de
    /// reconocimiento de imagenes para asi poder detectar incidentes en imagenes provistas por el usuario.
    image_classifier: ImageClassifier,
}

impl Camera {
    pub fn new(location: Location, id: u32) -> Result<Camera, CameraError> {
        let url = "https://vision.googleapis.com/v1/images:annotate".to_string();
        let incident_keywords_file_path = "./utils/incident_keywords";

        let image_classifier = ImageClassifier::new(url, incident_keywords_file_path)
            .map_err(|e| CameraError::AnnotationError(e.to_string()))?;

        let camera = Self {
            location,
            id,
            sleep_mode: true,
            image_classifier,
        };

        let dir_name = format!("./{}", camera.id);
        let path = PATH.to_string() + &dir_name;
        if let Err(e) = fs::create_dir_all(Path::new(path.as_str())) {
            return Err(CameraError::ArcMutexError(e.to_string()));
        }

        Ok(camera)
    }

    /// Utiliza su clasificador de imagenes para clasificar una imagen con un posible incidente.
    pub fn annotate_image(&self, image_path: &str) -> Result<bool, ProtocolError> {
        let classification_result = self
            .image_classifier
            .annotate_image(image_path)
            .map_err(|e| ProtocolError::AnnotationError(e.to_string()))?;

        Ok(classification_result)
    }

    pub fn get_location(&self) -> Location {
        self.location
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_sleep_mode(&self) -> bool {
        self.sleep_mode
    }

    pub fn set_sleep_mode(&mut self, sleep_mode: bool) {
        self.sleep_mode = sleep_mode;
    }

    pub fn write_to(&mut self, stream: &mut dyn Write) -> Result<(), CameraError> {
        write_u32(stream, &self.id).map_err(|_| CameraError::WriteError)?;
        write_string(stream, &self.location.get_latitude().to_string())
            .map_err(|_| CameraError::WriteError)?;
        write_string(stream, &self.location.get_longitude().to_string())
            .map_err(|_| CameraError::WriteError)?;
        write_bool(stream, &self.sleep_mode).map_err(|_| CameraError::WriteError)?;

        Ok(())
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<Camera, Error> {
        let url = "https://vision.googleapis.com/v1/images:annotate".to_string();
        let incident_keywords_file_path = "./utils/incident_keywords";

        let image_classifier = ImageClassifier::new(url, incident_keywords_file_path)
            .map_err(|e| CameraError::AnnotationError(e.to_string()))
            .expect("Fallo al crear clasificador de imagenes");

        let id = read_u32(stream)?;
        let latitude = read_string(stream)?;
        let longitude = read_string(stream)?;
        let sleep_mode = read_bool(stream)?;

        let lat = match latitude.parse::<f64>() {
            Ok(lat) => lat,
            Err(_) => {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid latitude",
                ))
            }
        };
        let long = match longitude.parse::<f64>() {
            Ok(long) => long,
            Err(_) => {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid longitude",
                ))
            }
        };
        Ok(Camera {
            location: Location::new(lat, long),
            id,
            sleep_mode,
            image_classifier,
        })
    }

    pub fn delete_directory(&self) -> Result<(), CameraError> {
        let dir_name = format!("./{}", self.id);
        let path = PATH.to_string() + &dir_name;
        fs::remove_dir_all(Path::new(&path)).map_err(|e| CameraError::DeleteDirError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new_camera() {
        let location = Location::new(1.0, 2.0);
        let camera = Camera::new(location, 1).unwrap();
        assert_eq!(camera.get_location(), location);
        assert_eq!(camera.get_id(), 1);
    }
    #[test]
    fn write_to_read_from() {
        let location = Location::new(1.0, 2.0);
        let mut camera = Camera::new(location, 1).unwrap();
        let mut buffer = Vec::new();
        camera.write_to(&mut buffer).unwrap();
        let mut buffer = &buffer[..];
        let camera_read = Camera::read_from(&mut buffer).unwrap();
        assert_eq!(camera, camera_read);
    }

    #[test]
    fn test_check_dir_creation() {
        let location = Location::new(1.0, 2.0);
        let camera = Camera::new(location, 1).unwrap();
        let dir_name = format!("./{}", camera.get_id());
        let path = "src/surveilling/cameras".to_string() + &dir_name;
        assert!(Path::new(path.as_str()).exists());
    }
    #[test]
    fn test_dir_creation_and_deletion() {
        let location = Location::new(1.0, 2.0);
        let camera = Camera::new(location, 1).unwrap();
        let dir_name = format!("./{}", camera.get_id());
        let path = "src/surveilling/cameras".to_string() + &dir_name;
        assert!(Path::new(path.as_str()).exists());
        camera.delete_directory().unwrap();
        assert!(!Path::new(path.as_str()).exists());
    }

    #[test]
    fn test_image_annotation_ok() {
        let location = Location::new(1.0, 2.0);
        let camera = Camera::new(location, 1).unwrap();

        let image_path_one = "./tests/ia_annotation_img/fire.png";
        let classification_result_one = camera.annotate_image(image_path_one).unwrap();

        let image_path_two = "./tests/ia_annotation_img/no_incident.png";
        let classification_result_two = camera.annotate_image(image_path_two).unwrap();

        assert!(classification_result_one);
        assert!(!classification_result_two);
    }
}