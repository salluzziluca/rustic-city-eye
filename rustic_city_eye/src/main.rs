extern crate gtk;

use std::cell::RefCell;
use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::sync::mpsc;

// use gtk::cairo::ImageSurface;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{glib, Window, WindowType};
use gtk::{Application, Box, Button, Entry, Label, Orientation};
use rustic_city_eye::monitoring::monitoring_app::MonitoringApp;
use rustic_city_eye::mqtt::protocol_error::ProtocolError;
use rustic_city_eye::surveilling::location::Location;
use webkit2gtk::{WebView, WebViewExt};
use webkit2gtk::gio;

fn main() -> Result<(), ProtocolError> {
    let app = Application::builder()
        .application_id("com.example.RusticCityEye")
        .build();

    app.connect_activate(|app| {
            let home_window = Window::new(WindowType::Toplevel);
            home_window.set_title("Rustic City Eye");
            home_window.set_default_size(2000, 1500);

            let vbox = Box::new(Orientation::Vertical, 5);

            let button = Button::with_label("Conectarse a un servidor");
            vbox.pack_start(&button, false, false, 0);

            let elements_container = Box::new(Orientation::Vertical, 5);
            vbox.pack_start(&elements_container, true, true, 0);

            button.connect_clicked(clone!(@weak button, @weak elements_container => move |_| {
                button.hide();

                let label = Label::new(Some("Nuevo elemento"));
                let host = Entry::new();
                host.set_placeholder_text(Some("Host: "));

                let port = Entry::new();
                port.set_placeholder_text(Some("Port: "));
                let user = Entry::new();
                user.set_placeholder_text(Some("User: "));
                let password = Entry::new();
                password.set_placeholder_text(Some("Password: "));

                let connect_btn = Button::with_label("Conectarse");
                elements_container.pack_start(&label, false, false, 0);
                elements_container.pack_start(&host, false, false, 0);
                elements_container.pack_start(&port, false, false, 0);
                elements_container.pack_start(&user, false, false, 0);
                elements_container.pack_start(&password, false, false, 0);
                elements_container.pack_start(&connect_btn, false, false, 0);

                connect_btn.connect_clicked(clone!(@weak host, @weak port, @weak user, @weak password, @weak elements_container => move |_| {
                    let host = host.text().to_string();
                    let port = port.text().to_string();
                    let username = user.text().to_string();
                    let password = password.text().to_string();
                    let mut args = Vec::new();
                    args.push(host);
                    args.push(port);
                    args.push(username);
                    args.push(password);

                    match MonitoringApp::new(args) {
                        Ok(mut monitoring_app) => {
                            elements_container.hide();
                            let (tx, rx) = mpsc::channel();
                            let _ = monitoring_app.run_client(rx);
                            //TODO: Aca deberiamos mostrar el mapa!!!
                            let webview = WebView::new();
                            elements_container.pack_start(&webview, true, true, 0);

                            let add_camera_btn = Button::with_label("Add camera");
                            elements_container.pack_start(&add_camera_btn, false, false, 0);


                            // Load Leaflet map
                            webview.load_uri("https://leafletjs.com/examples/quick-start/index.html");
                    
                            // Connect click event handler to get coordinates
                            let webview_clone = webview.clone();
                            let monitoring_app_ref = Rc::new(RefCell::new(monitoring_app));
                            
                            webview.connect_button_press_event(clone!(@strong monitoring_app_ref => move |_, event| {
                                if event.button() == 1 {
                                    let x = event.position().0;
                                    let y = event.position().1;
                                    
                                    // let monitoring_app_ref = Rc::new(RefCell::new(monitoring_app_ref));
                                    add_camera_btn.connect_clicked(clone!(@strong y, @strong x, @strong monitoring_app_ref => move |_| {
                                        let lat = y.to_string();
                                        let long = x.to_string();
                                        
                                        let camera_location = Location::new(lat, long);

                                        monitoring_app_ref.borrow_mut().add_camera(camera_location);
                                    }));

                                    webview_clone.run_javascript(
                                        &format!("window.map.setView(window.map.containerPointToLatLng([{}, {}]), window.map.getZoom());", x, y),
                                        None::<&gio::Cancellable>,
                                        |_result| (),
                                    );
                                }
                                false.into()
                            }));




                            let message = Entry::new();
                            message.set_placeholder_text(Some("Send message: "));
                            let send_btn = Button::with_label("Send");
                            elements_container.pack_start(&message, false, false, 0);
                            elements_container.pack_start(&send_btn, false, false, 0);

                            // let latitude = Entry::new();
                            // let longitude = Entry::new();

                            // latitude.set_placeholder_text(Some("Enter camera lat: "));
                            // longitude.set_placeholder_text(Some("Enter camera lat: "));
                            // elements_container.pack_start(&latitude, false, false, 0);
                            // elements_container.pack_start(&longitude, false, false, 0);
                            
                            // let add_camera_btn = Button::with_label("Add camera");
                            // elements_container.pack_start(&add_camera_btn, false, false, 0);

                            elements_container.show_all();

                            let tx_clone = tx.clone();
                            send_btn.connect_clicked(clone!(@weak message => move |_| {
                                let msg = message.text().to_string();
                                message.set_text("");

                                let _ = tx_clone.send(msg).map_err(|err| {
                                    Error::new(ErrorKind::Other, format!("Failed to send line: {}", err))
                                });
                            }));

                            // let monitoring_app_ref = Rc::new(RefCell::new(monitoring_app));
                            // add_camera_btn.connect_clicked(clone!(@weak latitude, @weak longitude, @strong monitoring_app_ref => move |_| {
                            //     let lat = latitude.text().to_string();
                            //     latitude.set_text("");
                            //     let long = longitude.text().to_string();
                            //     longitude.set_text("");

                            //     let camera_location = Location::new(lat, long);

                            //     monitoring_app_ref.borrow_mut().add_camera(camera_location);
                            // }));
                        },
                        Err(_) => todo!(),
                    }
                }));

                elements_container.show_all();
            }));

            // Agrega la caja a la ventana.
            home_window.add(&vbox);

            // Muestra todos los widgets en la ventana.
            home_window.show_all();

            // Configura la aplicación principal.
            home_window.set_application(Some(app));
        });

    app.run();

    Ok(())
}

// fn download_tile(url: &str) -> Result<ImageSurface, reqwest::Error> {
//     let response = reqwest::blocking::get(url)?;
//     let bytes = response.bytes()?;
//     let cursor = Cursor::new(bytes);

//     ImageSurface::create_from_png(cursor).map_err(|e| {
//         reqwest::Error::new(reqwest::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
//     })
// }