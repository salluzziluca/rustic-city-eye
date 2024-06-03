use egui::{Align2, RichText, Ui, Window};
use rustic_city_eye::{monitoring::monitoring_app::MonitoringApp, surveilling::location::Location};
use walkers::MapMemory;

use crate::{camera_view::CameraView, incident_view::IncidentView, MyMap};

/// Simple GUI to zoom in and out.
pub fn zoom(ui: &Ui, map_memory: &mut MapMemory) {
    Window::new("Map")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_BOTTOM, [10., -10.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("➕").heading()).clicked() {
                    let _ = map_memory.zoom_in();
                }

                if ui.button(RichText::new("➖").heading()).clicked() {
                    let _ = map_memory.zoom_out();
                }
            });
        });
}

pub fn add_camera_window(ui: &Ui, map: &mut MyMap, mut monitoring_app: &mut MonitoringApp) {
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

                        println!("mi monitoring: {:?}", monitoring_app);

                        map.cameras.push(
                            CameraView {
                                image: map.camera_icon.clone(),
                                position,
                                radius: map.camera_radius.clone(),
                            }
                        );

                    }


                }
            });
        });
}

pub fn add_incident_window(ui: &Ui, map: &mut MyMap) {
    Window::new("Add Incident")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::RIGHT_TOP, [-10., 50.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("🚨").heading()).clicked() {
                    if let Some(position) = map.click_watcher.clicked_at{
                        map.incidents.push(
                            IncidentView {
                                image: map.incident_icon.clone(),
                                position
                            }
                        );    
                    }
                }
            });
        });
    }
