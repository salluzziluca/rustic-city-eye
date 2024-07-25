mod camera_view;
mod drone_center_view;
mod drone_view;
mod incident_view;
mod plugins;
mod windows;

use camera_view::CameraView;
use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::{CentralPanel, RichText, TextStyle, TopBottomPanel};
use incident_view::IncidentView;
use plugins::*;
use rustic_city_eye::{
    monitoring::{incident::Incident, monitoring_app::MonitoringApp},
    surveilling::camera::Camera,
    utils::location::Location,
};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};
use walkers::{sources::OpenStreetMap, HttpTiles, Map, MapMemory, Position, Texture, Tiles};

use windows::*;

/// Este struct se utiliza para almacenar la informacion de las imagenes que se van a mostrar en el mapa
/// junto con su escala
/// Se utiliza para los iconos de camaras, incidentes, drones y centros de drones
/// La escala se actualiza teniendo en cuenta el zoom level
struct MyMap {
    tiles: Box<dyn Tiles>,
    map_memory: MapMemory,
    click_watcher: ClickWatcher,
    camera_icon: ImagesPluginData,
    cameras: HashMap<u32, CameraView>,
    camera_radius: ImagesPluginData,
    active_camera_radius: ImagesPluginData,
    incident_icon: ImagesPluginData,
    incidents: Vec<incident_view::IncidentView>,
    drones: HashMap<u32, drone_view::DroneView>,
    drone_icon: ImagesPluginData,
    drone_centers: Vec<drone_center_view::DroneCenterView>,
    drone_center_icon: ImagesPluginData,
    zoom_level: f32,
}
impl MyMap {
    /// Actualiza la posicion de los drones en el mapa
    fn update_drones(
        &mut self,
        updated_locations: Arc<Mutex<VecDeque<(u32, Location, Location)>>>,
    ) {
        match updated_locations.try_lock() {
            Ok(mut new_drone_locations) => {
                if !new_drone_locations.is_empty(){
                    println!("new_drone_locations: {:?}", new_drone_locations);
                } 
                loop {
                    if let Some((id, location, target_location)) = new_drone_locations.pop_front() {
                        if let Some(drone) = self.drones.get_mut(&id) {
                            drone.position = Position::from_lon_lat(location.long, location.lat);
                            drone.target_position =
                                Position::from_lon_lat(target_location.long, target_location.lat);
                            // } else {
                            //     let drone_view = drone_view::DroneView {
                            //         image: self.drone_icon.clone(),
                            //         position: Position::from_lon_lat(location.long, location.lat),
                            //         target_position: Position::from_lon_lat(target_location.long, target_location.lat),
                            //         clicked: false,
                            //     };
                            //     self.drones.insert(id, drone_view);
                            // }
                        }
                    } else {
                        break;
                    }
                }
            }
            Err(_) => {}
        };
        // for (id, location) in new_drone_locations {
        //     if let Some(drone) = self.drones.get_mut(&id) {
        //         drone.position = Position::from_lon_lat(location.0.long, location.0.lat);
        //         drone.target_position = Position::from_lon_lat(location.1.long, location.1.lat);
        //     }
        // }
        for drone in self.drones.values_mut() {
            drone.move_towards();
        }
    }

    /// Actualiza la posicion de las camaras en el mapa
    /// Si la camara esta en modo sleep, se muestra con un radio azul
    /// Si la camara esta activa, se muestra con un radio rojo
    ///
    fn update_cameras(&mut self, new_cameras: HashMap<u32, Camera>) {
        for (id, camera) in new_cameras {
            if let Some(camera_view) = self.cameras.get_mut(&id) {
                if !camera.get_sleep_mode() {
                    camera_view.radius = ImagesPluginData::new(
                        self.active_camera_radius.texture.clone(),
                        self.zoom_level,
                        self.active_camera_radius.y_scale,
                    );
                } else {
                    camera_view.radius = ImagesPluginData::new(
                        self.camera_radius.texture.clone(),
                        self.zoom_level,
                        self.camera_radius.y_scale,
                    );
                }
            }
        }
    }
    /// Actualiza la posicion de los incidentes en el mapa
    fn update_incidents(&mut self, incidents: Vec<Incident>) {
        let mut new_incident_view = vec![];
        for incident in incidents {
            let location = incident.get_location();

            let incident_view = IncidentView {
                image: self.incident_icon.clone(),
                position: Position::from_lon_lat(location.long, location.lat),
                clicked: false,
            };
            new_incident_view.push(incident_view);
        }

        self.incidents = new_incident_view;
    }
}

struct MyApp {
    username: String,
    password: String,
    ip: String,
    port: String,
    connected: bool,
    map: MyMap,
    monitoring_app: Option<MonitoringApp>,
    correct_username: bool,
    correct_password: bool,
    correct_ip: bool,
    correct_port: bool,
}
impl ImagesPluginData {
    /// recibe el zoom level inicial y la escala para cada una de las imagenes
    fn new(texture: Texture, initial_zoom_level: f32, original_scale: f32) -> Self {
        let scale = initial_zoom_level * original_scale;
        Self {
            texture,
            x_scale: scale,
            y_scale: scale,
        }
    }
}
impl MyApp {
    /// Muestra el formulario de inicio de sesion
    /// Si se presiona el boton de submit, se intenta conectar al servidor
    /// Si la conexion es exitosa, se muestra el mapa
    /// Al final de esta ventana se muestran los creditos
    fn handle_form(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("Username")
                        .size(20.0)
                        .color(egui::Color32::WHITE),
                );

                if self.correct_username {
                    ui.visuals_mut().extreme_bg_color = egui::Color32::BLACK;
                } else {
                    ui.visuals_mut().extreme_bg_color = egui::Color32::RED;
                }
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

                if self.correct_password {
                    ui.visuals_mut().extreme_bg_color = egui::Color32::BLACK;
                } else {
                    ui.visuals_mut().extreme_bg_color = egui::Color32::RED;
                }
                ui.add(
                    egui::TextEdit::singleline(&mut self.password)
                        .min_size(egui::vec2(100.0, 20.0))
                        .text_color(egui::Color32::WHITE)
                        .font(TextStyle::Body),
                );

                ui.label(RichText::new("IP").size(20.0).color(egui::Color32::WHITE));

                if self.correct_ip {
                    ui.visuals_mut().extreme_bg_color = egui::Color32::BLACK;
                } else {
                    ui.visuals_mut().extreme_bg_color = egui::Color32::RED;
                }
                ui.add(
                    egui::TextEdit::singleline(&mut self.ip)
                        .min_size(egui::vec2(100.0, 20.0))
                        .text_color(egui::Color32::WHITE)
                        .font(TextStyle::Body),
                );

                ui.label(RichText::new("Port").size(20.0).color(egui::Color32::WHITE));

                if self.correct_port {
                    ui.visuals_mut().extreme_bg_color = egui::Color32::BLACK;
                } else {
                    ui.visuals_mut().extreme_bg_color = egui::Color32::RED;
                }
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

                    self.correct_username = !self.username.is_empty();
                    self.correct_password = !self.password.is_empty();
                    self.correct_ip = !self.ip.is_empty();
                    self.correct_port = !self.port.is_empty();
                    match MonitoringApp::new(args) {
                        Ok(mut monitoring_app) => {
                            let _ = monitoring_app.run_client();
                            self.monitoring_app = Some(monitoring_app);
                            self.connected = true;
                        }
                        Err(e) => {
                            println!(
                                "La conexion ha fallado. Intenta conectarte nuevamente {}.",
                                e
                            );
                            self.username.clear();
                            self.password.clear();
                            self.ip.clear();
                            self.port.clear();
                        }
                    };
                }
            })
        });

        TopBottomPanel::bottom("credits_panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("rustic_city_eye 1C 2024 - Carranza Demarchi Giacobbe Salluzzi")
                        .size(15.0)
                        .color(egui::Color32::GRAY),
                );
            });
        });
    }
    /// Muestra el mapa
    /// Si se presiona el boton de zoom, se actualiza el zoom level
    /// Carga las diferentes ventanas de camaras, incidentes, drones y centros de drones
    fn handle_map(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if let Some(monitoring_app) = &self.monitoring_app {
            let lock = monitoring_app.connected.lock().unwrap();
            if !*lock {
                self.connected = false;
                self.username.clear();
                self.password.clear();
                self.ip.clear();
                self.port.clear();
            }
        }

        CentralPanel::default().show(ctx, |ui| {
            let last_clicked = self.map.click_watcher.clicked_at;

            // Dereference the Box<dyn Tiles> to access the underlying type
            let tiles_ref: &mut dyn Tiles = &mut *self.map.tiles;
            ui.add(
                Map::new(
                    Some(tiles_ref),
                    &mut self.map.map_memory,
                    Position::from_lon_lat(-58.368925, -34.61716),
                )
                .with_plugin(&mut self.map.click_watcher)
                .with_plugin(cameras(
                    &mut self.map.cameras,
                    self.map.zoom_level,
                    last_clicked,
                ))
                .with_plugin(incidents(
                    &mut self.map.incidents,
                    self.map.zoom_level,
                    last_clicked,
                ))
                .with_plugin(drones(
                    &mut self.map.drones,
                    self.map.zoom_level,
                    last_clicked,
                ))
                .with_plugin(drone_centers(
                    &mut self.map.drone_centers,
                    self.map.zoom_level,
                    last_clicked,
                )),
            );
            zoom(ui, &mut self.map.map_memory, &mut self.map.zoom_level);

            if let Some(monitoring_app) = &mut self.monitoring_app {
                let new_locations = monitoring_app.get_updated_drones();
                self.map.update_drones(new_locations);
                let new_cameras = monitoring_app.get_cameras();
                self.map.update_cameras(new_cameras);

                let incidents = monitoring_app.get_incidents();
                self.map.update_incidents(incidents);
                add_camera_window(ui, &mut self.map, monitoring_app);
                add_incident_window(ui, &mut self.map, monitoring_app);
                add_drone_window(ui, &mut self.map, monitoring_app);
                add_drone_center_window(ui, &mut self.map, monitoring_app);
                add_disconnect_window(ui, &mut self.map, monitoring_app, &mut self.connected);
                add_remove_window(ui, &mut self.map, monitoring_app)
            }
        });
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if !self.connected {
            self.monitoring_app = None;
            self.handle_form(ctx, _frame);
        } else {
            self.handle_map(ctx, _frame);
        }
        ctx.request_repaint();
    }
}
/// Funcion que se encarga de inicializar la aplicacion con sus texturas
/// inicializandola en su estado original
fn create_my_app(cc: &CreationContext<'_>) -> Box<dyn App> {
    egui_extras::install_image_loaders(&cc.egui_ctx.clone());
    let camera_bytes = include_bytes!("../assets/Camera.png");
    let camera_icon = match Texture::new(camera_bytes, &cc.egui_ctx) {
        Ok(t) => ImagesPluginData::new(t, 1.0, 0.1), // Initialize with zoom level 1.0
        Err(_) => todo!(),
    };

    let incident_bytes = include_bytes!("../assets/Incident.png");
    let incident_icon = match Texture::new(incident_bytes, &cc.egui_ctx) {
        Ok(t) => ImagesPluginData::new(t, 1.0, 0.15), // Initialize with zoom level 1.0
        Err(_) => todo!(),
    };

    let drone_bytes = include_bytes!("../assets/Drone.png");
    let drone_icon = match Texture::new(drone_bytes, &cc.egui_ctx) {
        Ok(t) => ImagesPluginData::new(t, 1.0, 0.06), // Initialize with zoom level 1.0
        Err(_) => todo!(),
    };

    let drone_center_bytes = include_bytes!("../assets/DroneCenter.png");
    let drone_center_icon = match Texture::new(drone_center_bytes, &cc.egui_ctx) {
        Ok(t) => ImagesPluginData::new(t, 1.0, 0.1), // Initialize with zoom level 1.0
        Err(_) => todo!(),
    };

    let circle_bytes = include_bytes!("../assets/circle.png");
    let circle_icon = match Texture::new(circle_bytes, &cc.egui_ctx) {
        Ok(t) => ImagesPluginData::new(t, 1.0, 0.2), // Initialize with zoom level 1.0
        Err(_) => todo!(),
    };

    let red_circle_bytes = include_bytes!("../assets/red_circle.png");

    let red_circle_icon = match Texture::new(red_circle_bytes, &cc.egui_ctx) {
        Ok(t) => ImagesPluginData::new(t, 1.0, 0.2), // Initialize with zoom level 1.0
        Err(_) => todo!(),
    };

    egui_extras::install_image_loaders(&cc.egui_ctx.clone());

    let tiles = Box::new(HttpTiles::new(OpenStreetMap, cc.egui_ctx.clone()));

    Box::new(MyApp {
        username: String::new(),
        password: String::new(),
        ip: String::new(),
        port: String::new(),
        connected: false,
        map: MyMap {
            tiles,
            map_memory: MapMemory::default(),
            click_watcher: ClickWatcher::default(),
            cameras: HashMap::new(),
            incidents: vec![],
            camera_icon,
            incident_icon,
            camera_radius: circle_icon,
            active_camera_radius: red_circle_icon,
            drones: HashMap::new(),
            drone_icon,
            drone_centers: vec![],
            drone_center_icon,
            zoom_level: 1.0,
        },
        monitoring_app: None,
        correct_username: true,
        correct_password: true,
        correct_ip: true,
        correct_port: true,
    })
}

// Entry point of the application, sets up window options and runs the main event loop
fn main() {
    let app_name = "Rustic City Eye";
    let win_options = NativeOptions {
        ..Default::default()
    };
    if let Err(e) = run_native(app_name, win_options, Box::new(|cc| Ok(create_my_app(cc)))) {
        eprintln!("Error: {}", e);
    }
}
