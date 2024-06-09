use egui::{Align2, RichText, Ui, Window};
use rustic_city_eye::{monitoring::monitoring_app::MonitoringApp, utils::location::Location};
use walkers::MapMemory;

use crate::{camera_view::CameraView, incident_view::IncidentView, drone_view::DroneView, MyMap};

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
                        monitoring_app.add_camera(location);
                        map.cameras.push(CameraView {
                            image: map.camera_icon.clone(),
                            position,
                            radius: map.camera_radius.clone(),
                        });
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
        .anchor(Align2::RIGHT_TOP, [-10., 90.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("📡").heading()).clicked() {
                    if let Some(position) = map.click_watcher.clicked_at {
                        // let location = Location::new(position.lat(), position.lon());
                        // monitoring_app.add_drone(location);
                        map.drones.push(DroneView {
                            image: map.drone_icon.clone(),
                            position,
                        });
                    }
                }
            });
        });
}

pub fn add_disconnect_window(ui: &Ui, map: &mut MyMap, monitoring_app: &mut MonitoringApp, connected: &mut bool) {
    Window::new("Disconnect")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::RIGHT_BOTTOM, [-10., -30.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("DIsconnect").heading()).clicked() {
                    // monitoring_app.disconnect();    No está implementado?
                    map.cameras.clear();
                    map.incidents.clear();
                    *connected = false;
                }
            });
        });
}
