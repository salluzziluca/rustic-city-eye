use egui::Response;
use walkers::{
    extras::{Image, Images, Texture},
    Plugin, Position, Projector,
};

use crate::camera_view::CameraView;
use crate::incident_view::IncidentView;

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
pub fn cameras(cameras: &mut Vec<CameraView>) -> impl Plugin {
    let mut images_vec = vec![];

    for camera in cameras {
        let mut radius = Image::new(camera.radius.texture.clone(), camera.position);
        radius.scale(camera.radius.x_scale, camera.radius.y_scale);
        images_vec.push(radius);

        let mut image = Image::new(camera.image.texture.clone(), camera.position);
        image.scale(camera.image.x_scale, camera.image.y_scale);
        images_vec.push(image);
    }

    Images::new(images_vec)
}

pub fn incidents(incidents: &mut Vec<IncidentView>) -> impl Plugin {
    let mut images_vec = vec![];

    for incident in incidents {
        let mut image = Image::new(incident.image.texture.clone(), incident.position);
        image.scale(incident.image.x_scale, incident.image.y_scale);
        images_vec.push(image);
    }

    Images::new(images_vec)
}
