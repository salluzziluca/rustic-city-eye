use egui::{Align2, RichText, Ui, Window};
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

pub fn add_camera_window(ui: &Ui, map: &mut MyMap) {
    Window::new("Add Camera")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::RIGHT_TOP, [-10., 10.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("📷").heading()).clicked() {
                    map.cameras.push(
                        CameraView {
                            image: map.camera_icon.clone(),
                            position: map.click_watcher.clicked_at.unwrap(),
                            radius: map.camera_radius.clone(),
                        }
                    );
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
                    map.incidents.push(
                        IncidentView {
                            image: map.incident_icon.clone(),
                            position: map.click_watcher.clicked_at.unwrap(),
                        }
                    );
                }
            });
        });
    }
