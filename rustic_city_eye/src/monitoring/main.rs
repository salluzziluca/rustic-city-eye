mod plugins;
mod windows;
mod camera_view;
mod incident_view;

use rustic_city_eye::monitoring::monitoring_app::MonitoringApp;
use walkers::{sources::OpenStreetMap, Map, MapMemory, Position, Texture, Tiles};
use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::{CentralPanel, RichText, TextStyle};
use plugins::{cameras, incidents, ClickWatcher, ImagesPluginData};
use windows::{add_camera_window, add_incident_window, zoom};
struct MyMap {
    tiles: Tiles, 
    map_memory: MapMemory,
    click_watcher: ClickWatcher,
    camera_icon: ImagesPluginData,
    cameras: Vec<camera_view::CameraView>,
    incident_icon:ImagesPluginData,
    incidents: Vec<incident_view::IncidentView>,
    camera_radius: ImagesPluginData,
}

struct MyApp {
    username: String,
    password: String,
    ip: String,
    port: String,
    connected: bool,
    map: MyMap,
    monitoring_app: Option<MonitoringApp>
}

impl MyApp{
    fn handle_form(&mut self,ctx: &eframe::egui::Context, _frame: &mut eframe::Frame){
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
                    let args = vec![self.username.clone(), self.password.clone(), self.ip.clone(), self.port.clone()];
                    match MonitoringApp::new(args) {
                        Ok(monitoring_app) => {
                            // let _ = monitoring_app.run_client();
                            self.monitoring_app = Some(monitoring_app);
                            self.connected = true;

                        },
                        Err(_) => {
                            println!("La conexion ha fallado. Intenta conectarte nuevamente.");
                            self.username.clear();
                            self.password.clear();
                            self.ip.clear();
                            self.port.clear();
                        },
                    };
                }
            })
        });

    }

    fn handle_map(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame){
        CentralPanel::default().show(ctx, |ui| {
            ui.add(Map::new(
                Some(&mut self.map.tiles),
                &mut self.map.map_memory,
                Position::from_lon_lat(-58.368925,-34.61716))
                .with_plugin(&mut self.map.click_watcher)
                .with_plugin(cameras(&mut self.map.cameras))
                .with_plugin(incidents(&mut self.map.incidents))

            );
            zoom(ui,&mut self.map.map_memory);
            if let Some(monitoring_app) = &mut self.monitoring_app {
                add_camera_window(ui,&mut self.map, monitoring_app);
            }
            add_incident_window(ui, &mut self.map);                
        });

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
        Ok(t) => {
            ImagesPluginData {
                texture: t,
                x_scale: 0.2,
                y_scale: 0.2,
            }
        },
        Err(_) => todo!(),
    };
    let incident_bytes = include_bytes!("assets/Incident.png");
    let incident_icon = match Texture::new(incident_bytes, &cc.egui_ctx){
        Ok(t) => {
            ImagesPluginData {
                texture: t,
                x_scale: 0.15,
                y_scale: 0.15,
            }
        }
        Err(_) => todo!(),
    };

    let circle_bytes = include_bytes!("assets/circle.png");
    let circle_icon = match Texture::new(circle_bytes, &cc.egui_ctx){
        Ok(t) => {
            ImagesPluginData {
                texture: t,
                x_scale: 0.2,
                y_scale: 0.2,
            }
        }
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
        },
        monitoring_app: None
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