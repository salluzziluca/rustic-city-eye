mod camera_view;
mod incident_view;
mod plugins;
mod windows;

use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::{CentralPanel, RichText, TextStyle};
use plugins::{cameras, incidents, ClickWatcher};
use rustic_city_eye::monitoring::monitoring_app::MonitoringApp;
use walkers::{sources::OpenStreetMap, Map, MapMemory, Position, Texture, Tiles};
use windows::{add_camera_window, add_incident_window, zoom};
#[derive(Clone)]
struct ImagesPluginData {
    texture: Texture,
    x_scale: f32,
    y_scale: f32,
    original_scale: f32,
}

struct MyMap {
    tiles: Tiles,
    map_memory: MapMemory,
    click_watcher: ClickWatcher,
    camera_icon: ImagesPluginData,
    cameras: Vec<camera_view::CameraView>,
    incident_icon: ImagesPluginData,
    incidents: Vec<incident_view::IncidentView>,
    camera_radius: ImagesPluginData,
    zoom_level: f32,
}

struct MyApp {
    username: String,
    password: String,
    ip: String,
    port: String,
    connected: bool,
    map: MyMap,
    monitoring_app: Option<MonitoringApp>,
}

impl ImagesPluginData {
    /// recibe el zoom level inicial y la escala para cada una de las imagenes
    fn new(texture: Texture, initial_zoom_level: f32, original_scale: f32) -> Self {
        let scale = initial_zoom_level * original_scale;
        Self {
            texture,
            x_scale: scale,
            y_scale: scale,
            original_scale,
        }
    }

    ///calcula la escala segun la escala original y el nuevo zoom level
    fn calculate_scale(&mut self, zoom_level: f32) -> f32 {
        self.original_scale * zoom_level
    }
    /// updatea la escala de la imagen, modificando los ejes segun la nueva escala
    /// que depende del zoom
    fn update_scale(&mut self, ctx: &egui::Context, zoom_level: f32) {
        let scale = Self::calculate_scale(self, zoom_level);
        self.x_scale = scale;
        self.y_scale = scale;

        // Mark UI as dirty to trigger refresh
        ctx.request_repaint();
    }
}
impl MyApp {
    fn handle_form(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("Username")
                        .size(20.0)
                        .color(egui::Color32::WHITE),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.username)
                        .min_size(egui::vec2(100.0, 20.0))
                        .text_color(egui::Color32::WHITE)
                        .font(TextStyle::Body),
                );

                ui.label(
                    RichText::new("Password")
                        .size(20.0)
                        .color(egui::Color32::WHITE),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.password)
                        .min_size(egui::vec2(100.0, 20.0))
                        .text_color(egui::Color32::WHITE)
                        .font(TextStyle::Body),
                );

                ui.label(RichText::new("IP").size(20.0).color(egui::Color32::WHITE));
                ui.add(
                    egui::TextEdit::singleline(&mut self.ip)
                        .min_size(egui::vec2(100.0, 20.0))
                        .text_color(egui::Color32::WHITE)
                        .font(TextStyle::Body),
                );

                ui.label(RichText::new("Port").size(20.0).color(egui::Color32::WHITE));
                ui.add(
                    egui::TextEdit::singleline(&mut self.port)
                        .min_size(egui::vec2(100.0, 20.0))
                        .text_color(egui::Color32::WHITE)
                        .font(TextStyle::Body),
                );

                if ui.button("Submit").clicked() {
                    let args = vec![
                        self.username.clone(),
                        self.password.clone(),
                        self.ip.clone(),
                        self.port.clone(),
                    ];
                    match MonitoringApp::new(args) {
                        Ok(mut monitoring_app) => {
                            let _ = monitoring_app.run_client();
                            self.monitoring_app = Some(monitoring_app);
                            self.connected = true;
                        }
                        Err(_) => {
                            println!("La conexion ha fallado. Intenta conectarte nuevamente.");
                            self.username.clear();
                            self.password.clear();
                            self.ip.clear();
                            self.port.clear();
                        }
                    };
                }
            })
        });
    }

    fn handle_map(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.add(
                Map::new(
                    Some(&mut self.map.tiles),
                    &mut self.map.map_memory,
                    Position::from_lon_lat(-58.368925, -34.61716),
                )
                .with_plugin(&mut self.map.click_watcher)
                .with_plugin(cameras(&mut self.map.cameras))
                .with_plugin(incidents(&mut self.map.incidents)),
            );
            zoom(ui, &mut self.map.map_memory, &mut self.map.zoom_level);

            // Get the current zoom level, however it's done in your code
            let current_zoom_level = self.get_current_zoom_level();

            // Update the scale for each image based on the current zoom level
            self.map.camera_icon.update_scale(ctx, current_zoom_level);
            self.map.incident_icon.update_scale(ctx, current_zoom_level);
            self.map.camera_radius.update_scale(ctx, current_zoom_level);

            if let Some(monitoring_app) = &mut self.monitoring_app {
                add_camera_window(ui, &mut self.map, monitoring_app);
                add_incident_window(ui, &mut self.map, monitoring_app);
            }
        });
    }

    /// Busca el zoom level actual
    fn get_current_zoom_level(&self) -> f32 {
        self.map.zoom_level
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if !self.connected {
            self.handle_form(ctx, _frame);
        } else {
            self.handle_map(ctx, _frame);
        }
    }
}

fn create_my_app(cc: &CreationContext<'_>) -> Box<dyn App> {
    egui_extras::install_image_loaders(&cc.egui_ctx);
    let camera_bytes = include_bytes!("assets/Camera.png");
    let camera_icon = match Texture::new(camera_bytes, &cc.egui_ctx) {
        Ok(t) => ImagesPluginData::new(t, 1.0, 0.1), // Initialize with zoom level 1.0
        Err(_) => todo!(),
    };
    let incident_bytes = include_bytes!("assets/Incident.png");
    let incident_icon = match Texture::new(incident_bytes, &cc.egui_ctx) {
        Ok(t) => ImagesPluginData::new(t, 1.0, 0.15), // Initialize with zoom level 1.0
        Err(_) => todo!(),
    };

    let circle_bytes = include_bytes!("assets/circle.png");
    let circle_icon = match Texture::new(circle_bytes, &cc.egui_ctx) {
        Ok(t) => ImagesPluginData::new(t, 1.0, 0.2), // Initialize with zoom level 1.0
        Err(_) => todo!(),
    };

    Box::new(MyApp {
        username: String::new(),
        password: String::new(),
        ip: String::new(),
        port: String::new(),
        connected: false,
        map: MyMap {
            tiles: Tiles::new(OpenStreetMap, cc.egui_ctx.clone()),
            map_memory: MapMemory::default(),
            click_watcher: ClickWatcher::default(),
            cameras: vec![],
            incidents: vec![],
            camera_icon,
            incident_icon,
            camera_radius: circle_icon,
            zoom_level: 1.0,
        },
        monitoring_app: None,
    })
}

fn main() {
    let app_name = "Rustic City Eye";
    let win_options = NativeOptions {
        ..Default::default()
    };
    if let Err(e) = run_native(app_name, win_options, Box::new(|cc| create_my_app(cc))) {
        eprintln!("Error: {}", e);
    }
}
