use egui::{Align2, RichText, Ui, Window};
use rustic_city_eye::{monitoring::monitoring_app::MonitoringApp, utils::location::Location};
use walkers::MapMemory;

use crate::{
    camera_view::CameraView, drone_center_view::DroneCenterView, drone_view::DroneView,
    incident_view::IncidentView, MyMap,
};

/// Simple GUI to zoom in and out.
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
                        println!("Camera added: {:?}", position);
                    }
                }
            });
        });
}

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
                        monitoring_app.add_incident(location);

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
                        monitoring_app.add_drone_center(location);
                        map.drone_centers.push(DroneCenterView {
                            image: map.drone_center_icon.clone(),
                            position,
                            clicked: false,
                        });
                    }
                }
            });
        });
}

pub fn add_drone_window(ui: &Ui, map: &mut MyMap, monitoring_app: &mut MonitoringApp) {
    Window::new("Add Drone")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::RIGHT_TOP, [-10., 170.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("🛸").heading()).clicked() {
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
                    let _ = monitoring_app.disconnect();
                    map.cameras.clear();
                    map.incidents.clear();
                    *connected = false;
                }
            });
        });
}

pub fn add_remove_window(ui: &Ui, map: &mut MyMap, _monitoring_app: &mut MonitoringApp) {
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
                    map.cameras.retain(|_id, camera| !camera.clicked);

                    map.incidents.retain(|incident| !incident.clicked);

                    map.drones.retain(|_id, drone| !drone.clicked);
                 
                    map.drone_centers
                        .retain(|drone_center| !drone_center.clicked);
                }
            });
        });
}
