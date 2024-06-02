use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::{CentralPanel, Image, RichText, TextStyle};

struct MyApp {
    usermane: String,
    ip: String,
    port: String,
    connected: bool,
    last_clicked_pos: egui::Pos2
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
                ui.image(egui::include_image!("assets/Map.png"));

                if ui.input(|input| input.pointer.primary_clicked()) {
                    if let Some(pos) = ui.input(|input| input.pointer.interact_pos()){
                        println!("Clicked at {:?}", pos);
                        self.last_clicked_pos = pos;
                    }
                }
                if  self.last_clicked_pos != egui::Pos2::ZERO {
                    ui.painter().circle(self.last_clicked_pos, 10.0, egui::Color32::RED, egui::Stroke::new(1.0, egui::Color32::WHITE));
                }

            });

        }
    }
    
}

fn create_my_app(_cc: &CreationContext<'_>) -> Box<dyn App> {
    egui_extras::install_image_loaders(&_cc.egui_ctx);

    Box::new(MyApp {
        usermane: String::new(),
        ip: String::new(),
        port: String::new(),
        connected: false,
        last_clicked_pos: egui::Pos2::ZERO,
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
