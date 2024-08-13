use std::{collections::{HashMap, VecDeque}, sync::{Arc, Mutex}};

use utils::{camera::Camera, incident::Incident, location::Location};
use walkers::{MapMemory, Position, Tiles};

use crate::{camera_view::CameraView, drone_center_view, drone_view, incident_view::{self, IncidentView}, plugings::{ClickWatcher, ImagesPluginData}};

/// Este struct se utiliza para almacenar la informacion de las imagenes que se van a mostrar en el mapa
/// junto con su escala
/// Se utiliza para los iconos de camaras, incidentes, drones y centros de drones
/// La escala se actualiza teniendo en cuenta el zoom level
pub struct MyMap {
    pub tiles: Box<dyn Tiles>,
    pub map_memory: MapMemory,
    pub click_watcher: ClickWatcher, 
    pub camera_icon: ImagesPluginData, 
    pub cameras: HashMap<u32, CameraView>, 
    pub camera_radius: ImagesPluginData, 
    pub active_camera_radius: ImagesPluginData, 
    pub incident_icon: ImagesPluginData, 
    pub incidents: Vec<incident_view::IncidentView>, 
    pub drones: HashMap<u32, drone_view::DroneView>, 
    pub drone_icon: ImagesPluginData, 
    pub drone_centers: HashMap<u32, drone_center_view::DroneCenterView>, 
    pub drone_center_icon: ImagesPluginData, 
    pub zoom_level: f32, 
}
impl MyMap {
    /// Actualiza la posicion de los drones en el mapa
    pub fn update_drones(
        &mut self,
        updated_locations: Arc<Mutex<VecDeque<(u32, Location, Location)>>>,
    ) {
        if let Ok(mut new_drone_locations) = updated_locations.try_lock() {

            while let Some((id, location, target_location)) = new_drone_locations.pop_front() {
                if let Some(drone) = self.drones.get_mut(&id) {
                    drone.position = Position::from_lon_lat(location.long, location.lat);
                    drone.target_position =
                        Position::from_lon_lat(target_location.long, target_location.lat);
                }
            }
        };
        for drone in self.drones.values_mut() {
            drone.move_towards();
        }
    }

    /// Actualiza la posicion de las camaras en el mapa
    /// Si la camara esta en modo sleep, se muestra con un radio azul
    /// Si la camara esta activa, se muestra con un radio rojo
    ///
    pub fn update_cameras(&mut self, new_cameras: HashMap<u32, Camera>) {
        for (id, camera) in new_cameras {
            if let Some(camera_view) = self.cameras.get_mut(&id) {
                camera_view.active = !camera.get_sleep_mode();
            }
        }
    }
    /// Actualiza la posicion de los incidentes en el mapa
    pub fn update_incidents(&mut self, incidents: Vec<Incident>) {
        let mut new_incident_view = vec![];
        for incident in incidents {
            let location = incident.get_location();

            let incident_view = IncidentView {
                image: self.incident_icon.clone(),
                position: Position::from_lon_lat(location.long, location.lat),
                clicked: false,
            };
            new_incident_view.push(incident_view);
        }

        self.incidents = new_incident_view;
    }
}