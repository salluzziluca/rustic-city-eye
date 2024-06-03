use egui::{ColorImage, Response};
use walkers::{
    Plugin, Position, Projector, 
    extras::{Images, Image, Texture},
};


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
            painter.circle_filled(projector.project(position).to_pos2(), 15.0, egui::Color32::RED);
        }
    }
}