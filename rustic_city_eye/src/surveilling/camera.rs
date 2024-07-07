use std::{
    fs,
    io::{Error, Read, Write},
    path::Path,
};

use serde::Deserialize;

use crate::utils::{
    location::Location,
    reader::{read_bool, read_string, read_u32},
};

use super::camera_error::CameraError;

use crate::utils::writer::{write_bool, write_string, write_u32};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Camera {
    location: Location,
    id: u32,
    sleep_mode: bool,
}

impl Camera {
    pub fn new(location: Location, id: u32) -> Camera {
        let camera = Self {
            location,
            id,
            sleep_mode: true,
        };

        let dir_name = format!("./{}", camera.id);
        let path = "src/surveilling/cameras".to_string() + &dir_name;
        if let Err(e) = fs::create_dir_all(Path::new(path.as_str())) {
            eprintln!(
                "Error al crear el directorio para la cámara {}: {}",
                camera.id, e
            );
        }

        camera
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new_camera() {
        let location = Location::new(1.0, 2.0);
        let camera = Camera::new(location, 1);
        assert_eq!(camera.get_location(), location);
        assert_eq!(camera.get_id(), 1);
    }
    #[test]
    fn write_to_read_from() {
        let location = Location::new(1.0, 2.0);
        let mut camera = Camera::new(location, 1);
        let mut buffer = Vec::new();
        camera.write_to(&mut buffer).unwrap();
        let mut buffer = &buffer[..];
        let camera_read = Camera::read_from(&mut buffer).unwrap();
        assert_eq!(camera, camera_read);
    }

    #[test]
    fn test_check_dir_creation() {
        let location = Location::new(1.0, 2.0);
        let camera = Camera::new(location, 1);
        let dir_name = format!("./{}", camera.get_id());
        let path = "src/surveilling/cameras".to_string() + &dir_name;
        assert!(Path::new(path.as_str()).exists());
    }
}
