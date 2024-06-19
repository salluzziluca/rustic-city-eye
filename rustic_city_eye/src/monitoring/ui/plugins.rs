use egui::Response;
use walkers::{
    extras::{Image, Images, Texture},
    Plugin, Position, Projector,
};

use crate::{camera_view::CameraView, drone_view::DroneView};
use crate::{drone_center_view::DroneCenterView, incident_view::IncidentView};

#[derive(Default, Clone)]
pub struct ClickWatcher {
    pub clicked_at: Option<Position>,
}

impl ClickWatcher {
    pub fn show_position(&self, ui: &egui::Ui) {
        if let Some(clicked_at) = self.clicked_at {
            egui::Window::new("Clicked Position")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .anchor(egui::Align2::CENTER_BOTTOM, [0., -10.])
                .show(ui.ctx(), |ui| {
                    ui.label(format!("{:.04} {:.04}", clicked_at.lon(), clicked_at.lat()))
                        .on_hover_text("last clicked position");
                });
        }
    }
}

impl Plugin for &mut ClickWatcher {
    fn run(&mut self, response: &Response, painter: egui::Painter, projector: &Projector) {
        if !response.changed() && response.clicked_by(egui::PointerButton::Primary) {
            self.clicked_at = response
                .interact_pointer_pos()
                .map(|p| projector.unproject(p - response.rect.center()));
        }

        if let Some(position) = self.clicked_at {
            painter.circle_filled(
                projector.project(position).to_pos2(),
                15.0,
                egui::Color32::GRAY,
            );
        }
    }
}

/// Helper structure for the `Images` plugin.
#[derive(Clone)]
pub struct ImagesPluginData {
    pub texture: Texture,
    pub x_scale: f32,
    pub y_scale: f32,
}

// Creates a built-in `Images` plugin with an example image.
pub fn cameras(
    cameras: &mut Vec<CameraView>,
    zoom_level: f32,
    last_clicked: Option<Position>,
) -> impl Plugin {
    let mut images_vec = vec![];

    for camera in cameras {
        let mut radius = Image::new(camera.radius.texture.clone(), camera.position);
        radius.scale(
            camera.radius.x_scale * zoom_level,
            camera.radius.y_scale * zoom_level,
        );
        images_vec.push(radius);

        let mut image = Image::new(camera.image.texture.clone(), camera.position);
        if camera.select(last_clicked) {
            image.scale(
                camera.image.x_scale * zoom_level * 1.5,
                camera.image.y_scale * zoom_level * 1.5,
            );
        } else {
            image.scale(
                camera.image.x_scale * zoom_level,
                camera.image.y_scale * zoom_level,
            );
        }
        images_vec.push(image);
    }
    Images::new(images_vec)
}

pub fn incidents(
    incidents: &mut Vec<IncidentView>,
    zoom_level: f32,
    last_clicked: Option<Position>,
) -> impl Plugin {
    let mut images_vec = vec![];

    for incident in incidents {
        let mut image = Image::new(incident.image.texture.clone(), incident.position);

        if incident.select(last_clicked) {
            image.scale(
                incident.image.x_scale * zoom_level * 1.5,
                incident.image.y_scale * zoom_level * 1.5,
            );
        } else {
            image.scale(
                incident.image.x_scale * zoom_level,
                incident.image.y_scale * zoom_level,
            );
        }
        images_vec.push(image);
    }
    Images::new(images_vec)
}

pub fn drones(
    drones: &mut Vec<DroneView>,
    zoom_level: f32,
    last_clicked: Option<Position>,
) -> impl Plugin {
    let mut images_vec = vec![];

    for drone in drones {
        let mut image = Image::new(drone.image.texture.clone(), drone.position);

        if drone.select(last_clicked) {
            image.scale(
                drone.image.x_scale * zoom_level * 1.5,
                drone.image.y_scale * zoom_level * 1.5,
            );
        } else {
            image.scale(
                drone.image.x_scale * zoom_level,
                drone.image.y_scale * zoom_level,
            );
        }
        images_vec.push(image);
    }
    Images::new(images_vec)
}

pub fn drone_centers(
    drone_centers: &mut Vec<DroneCenterView>,
    zoom_level: f32,
    last_clicked: Option<Position>,
) -> impl Plugin {
    let mut images_vec = vec![];

    for drone_center in drone_centers {
        let mut image = Image::new(drone_center.image.texture.clone(), drone_center.position);

        if drone_center.select(last_clicked) {
            image.scale(
                drone_center.image.x_scale * zoom_level * 1.5,
                drone_center.image.y_scale * zoom_level * 1.5,
            );
        } else {
            image.scale(
                drone_center.image.x_scale * zoom_level,
                drone_center.image.y_scale * zoom_level,
            );
        }
        images_vec.push(image);
    }
    Images::new(images_vec)
}
