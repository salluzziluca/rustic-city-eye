mod plugins;
mod windows;

use walkers::{sources::OpenStreetMap, Map, MapMemory, Plugin, Position, Texture, Tiles};
use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::{CentralPanel, Image, RichText, TextStyle};
use plugins::ClickWatcher;
use windows::{add_camera_window, add_incident_window, zoom};

struct MyMap {
    tiles: Tiles, 
    map_memory: MapMemory,
    click_watcher: ClickWatcher,
    cameras: Vec<Position>,
    incidents: Vec<Position>,
    // camera_icon: Texture,
    // incident_icon:Texture,
}

struct MyApp {
    usermane: String,
    ip: String,
    port: String,
    connected: bool,
    map: MyMap,
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
                    egui::TextEdit::singleline(&mut self.usermane)
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
                    println!(
                        "Username: {}, IP: {}, Port: {}",
                        self.usermane, self.ip, self.port
                    );
                    self.connected = true;
                }
            })
        });

    }


}

impl App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if !self.connected {
            self.handle_form(ctx, _frame);
        } else {
            CentralPanel::default().show(ctx, |ui| {
                ui.add(Map::new(
                    Some(&mut self.map.tiles),
                    &mut self.map.map_memory,
                    Position::from_lon_lat(-58.368925,-34.61716)
                ).with_plugin(&mut self.map.click_watcher)
                );
                zoom(ui,&mut self.map.map_memory);
                add_camera_window(ui,&mut self.map);
                add_incident_window(ui, &mut self.map);                
                });

        }
    }
    
}

fn create_my_app(cc: &CreationContext<'_>) -> Box<dyn App> {
    egui_extras::install_image_loaders(&cc.egui_ctx);

    Box::new(MyApp {
        usermane: String::new(),
        ip: String::new(),
        port: String::new(),
        connected: false,
        map: MyMap {
            tiles: Tiles::new(OpenStreetMap, cc.egui_ctx.clone()),
            map_memory: MapMemory::default(),
            click_watcher: ClickWatcher::default(),
            cameras: vec![],
            incidents: vec![],
        },
    })
}

fn main() {
    let app_name = "City_eye";
    let win_options = NativeOptions {
        ..Default::default()
    };
    if let Err(e) = run_native(app_name, win_options, Box::new(|cc| create_my_app(cc))) {
        eprintln!("Error: {}", e);
    }
}

