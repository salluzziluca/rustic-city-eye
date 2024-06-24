use std::io::{Error, Read, Write};

use serde::Deserialize;

use crate::utils::{location::Location, reader::{read_bool, read_string, read_u32}};

use super::camera_error::CameraError;

use crate::utils::writer::{write_bool, write_string, write_u32};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(dead_code)]
pub struct Camera {
    location: Location,
    id: u32,
    sleep_mode: bool,
}

impl Camera {
    pub fn new(location: Location, id: u32) -> Camera {
        Self {
            location,
            id,
            sleep_mode: true,
        }
    }

    pub fn get_location(&self) -> Location {
        self.location.clone()
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
        let _ = write_u32(stream, &self.id).map_err(|_| CameraError::WriteError);
        let _ = write_string(stream, &self.location.get_latitude().to_string())
            .map_err(|_| CameraError::WriteError);
        let _ = write_string(stream, &self.location.get_longitude().to_string())
            .map_err(|_| CameraError::WriteError);
        let _ = write_bool(stream, &self.sleep_mode).map_err(|_| CameraError::WriteError);

        Ok(())
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<Camera, Error> {
        let id = read_u32(stream)?;
        let latitude = read_string(stream)?;
        let longitude = read_string(stream)?;
        let sleep_mode = read_bool(stream)?;

        Ok(Camera {
            location: Location::new(latitude.parse().unwrap(), longitude.parse().unwrap()),
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
        let camera = Camera::new(location.clone(), 1);
        assert_eq!(camera.get_location(), location);
        assert_eq!(camera.get_id(), 1);
    }
}
