use std::fs;

use egui::{Align2, RichText, Ui, Window};
use rustic_city_eye::{
    drones::drones_central_config::DronesCentralConfig,
    monitoring::monitoring_app::{self, MonitoringApp},
    surveilling::cameras_config::CamerasConfig,
    utils::location::Location,
};
use walkers::MapMemory;

use crate::{
    camera_view::CameraView, drone_center_view::DroneCenterView, drone_view::DroneView,
    incident_view::IncidentView, MyMap,
};

/// Se updatea el zoom level con cada click en los botones de zoom
pub fn zoom(ui: &Ui, map_memory: &mut MapMemory, zoom_level: &mut f32) {
    Window::new("Map")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_BOTTOM, [10., -10.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("➕").heading()).clicked() {
                    let _ = map_memory.zoom_in();
                    *zoom_level *= 2.;
                }

                if ui.button(RichText::new("➖").heading()).clicked() {
                    let _ = map_memory.zoom_out();
                    *zoom_level /= 2.;
                }
            });
        });
}
/// Se añade una ventana para agregar una cámara
/// Al tocar el boton, se añade una camara en la posicion anteriormente seleccionada,
/// y se llama a la funcion `monitoring_app.add_camera` para añadir la camara al sistema de monitoreo.
/// Se añade la camara al mapa y se imprime un mensaje en consola.
pub fn add_camera_window(ui: &Ui, map: &mut MyMap, monitoring_app: &mut MonitoringApp) {
    Window::new("Add Camera")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::RIGHT_TOP, [-10., 10.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("📷").heading()).clicked() {
                    if let Some(position) = map.click_watcher.clicked_at {
                        let location = Location::new(position.lat(), position.lon());
                        let id = match monitoring_app.add_camera(location) {
                            Ok(result) => result,
                            Err(e) => {
                                println!("Error adding camera: {}", e);
                                return;
                            }
                        };
                        map.cameras.insert(
                            id,
                            CameraView {
                                image: map.camera_icon.clone(),
                                position,
                                radius: map.camera_radius.clone(),
                                clicked: false,
                            },
                        );
                    }
                }
            });
        });
}

/// Se añade una ventana para agregar un incidente
/// Al tocar el boton, se añade un incidente en la posicion anteriormente seleccionada,
/// y se llama a la funcion `monitoring_app.add_incident` para añadir el incidente al sistema de monitoreo.
/// Se añade el incidente al mapa.
pub fn add_incident_window(ui: &Ui, map: &mut MyMap, monitoring_app: &mut MonitoringApp) {
    Window::new("Add Incident")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::RIGHT_TOP, [-10., 50.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("🚨").heading()).clicked() {
                    if let Some(position) = map.click_watcher.clicked_at {
                        let location = Location::new(position.lat(), position.lon());
                        let _ = monitoring_app.add_incident(location);

                        map.incidents.push(IncidentView {
                            image: map.incident_icon.clone(),
                            position,
                            clicked: false,
                        });
                    }
                }
            });
        });
}

/// Se añade una ventana para agregar un centro de drones
/// Al tocar el boton, se añade un centro de drones en la posicion anteriormente seleccionada,
/// y se llama a la funcion `monitoring_app.add_drone_center` para añadir el centro de drones al sistema de monitoreo.
/// Se añade el centro de drones al mapa.
pub fn add_drone_center_window(ui: &Ui, map: &mut MyMap, monitoring_app: &mut MonitoringApp) {
    Window::new("Add Drone Center")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::RIGHT_TOP, [-10., 90.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("📡").heading()).clicked() {
                    if let Some(position) = map.click_watcher.clicked_at {
                        let location = Location::new(position.lat(), position.lon());
                        let id = monitoring_app.add_drone_center(location);
                        map.drone_centers.insert(
                            id,
                            DroneCenterView {
                                image: map.drone_center_icon.clone(),
                                position,
                                clicked: false,
                            },
                        );
                    }
                }
            });
        });
}
/// Se añade una ventana para agregar un drone
/// Al tocar el boton, se añade un dron en la posicion anteriormente seleccionada,
/// y se llama a la funcion `monitoring_app.add_drone` para añadir el drone al sistema de monitoreo.
/// Se añade el drone al mapa.
pub fn add_drone_window(ui: &Ui, map: &mut MyMap, monitoring_app: &mut MonitoringApp) {
    Window::new("Add Drone")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::RIGHT_TOP, [-10., 170.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("🚓").heading()).clicked() {
                    if let Some(position) = map.click_watcher.clicked_at {
                        let location = Location::new(position.lat(), position.lon());
                        let id = match monitoring_app.add_drone(location, 0) {
                            Ok(result) => result,
                            Err(e) => {
                                println!("Error adding drone: {}", e);
                                return;
                            }
                        };

                        map.drones.insert(
                            id,
                            DroneView {
                                image: map.drone_icon.clone(),
                                position,
                                clicked: false,
                                // id,
                            },
                        );
                    }
                }
            });
        });
}

/// Se añade una ventana para desconectar el sistema de monitoreo
/// Al tocar el boton, se llama a la funcion `monitoring_app.disconnect` para desconectar el sistema de monitoreo.

pub fn add_disconnect_window(
    ui: &Ui,
    map: &mut MyMap,
    monitoring_app: &mut MonitoringApp,
    connected: &mut bool,
) {
    Window::new("Disconnect")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::RIGHT_BOTTOM, [-10., -30.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("Disconnect").heading()).clicked() {
                    match monitoring_app.disconnect() {
                        Ok(_) => println!("Disconnected"),
                        Err(e) => println!("Error disconnecting: {}", e),
                    };
                    map.cameras.clear();
                    map.incidents.clear();
                    map.drone_centers.clear();
                    map.drones.clear();
                    *connected = false;
                }
            });
        });
}
/// Se añade una ventana para eliminar entidades del sistema de monitoreo
/// Al tocar el boton, se eliminan laa entidad que ha sido seleccionada en el mapa.
pub fn add_remove_window(ui: &Ui, map: &mut MyMap, monitoring_app: &mut MonitoringApp) {
    Window::new("Remove")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::RIGHT_TOP, [-10., 130.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui
                    .button(RichText::new("🗑").heading().color(egui::Color32::RED))
                    .clicked()
                {
                    for (id, camera) in map.cameras.iter() {
                        if camera.clicked {
                            println!("Removing camera {}", id);
                            //delete the dir rustic_city_eye/src/surveilling/cameras./id
                            fs::remove_dir_all(format!("src/surveilling/cameras./{}", id)).unwrap();
                            CamerasConfig::remove_camera_from_file(*id).unwrap();

                            break;
                        }
                    }

                    for (id, drone) in map.drones.iter() {
                        if drone.clicked {
                            println!("Removing drone {}", id);
                            DronesCentralConfig::remove_drone_from_json(*id).unwrap();
                            monitoring_app.disconnect_drone_by_id(*id).unwrap();
                            break;
                        }
                    }

                    for (id, drone_center) in map.drone_centers.iter() {
                        if drone_center.clicked {
                            println!("Removing drone center {}", id);
                            DronesCentralConfig::remove_central_from_json(*id).unwrap();

                            break;
                        }
                    }
                    map.cameras.retain(|_id, camera| !camera.clicked);

                    map.incidents.retain(|incident| !incident.clicked);

                    map.drones.retain(|_id, drone| !drone.clicked);

                    map.drone_centers
                        .retain(|_id, drone_center| !drone_center.clicked);
                }
            });
        });
}
