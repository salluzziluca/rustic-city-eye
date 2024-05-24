extern crate gtk;

use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{glib, Window, WindowType};
use gtk::{Application, Box, Button, Entry, Label, Orientation};
use webkit2gtk::gio;
use webkit2gtk::{WebView, WebViewExt};

use std::cell::RefCell;
use std::rc::Rc;

use rustic_city_eye::monitoring::monitoring_app::MonitoringApp;
use rustic_city_eye::mqtt::protocol_error::ProtocolError;
use rustic_city_eye::surveilling::location::Location;

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

                let label = Label::new(Some("Ingrese datos del servidor"));
                let host = Entry::new();
                host.set_placeholder_text(Some("Host: "));

                let port = Entry::new();
                port.set_placeholder_text(Some("Port: "));
                let user = Entry::new();
                user.set_placeholder_text(Some("Username: "));
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
                    let args = vec![host.text().to_string(), port.text().to_string(), user.text().to_string(), password.text().to_string()];

                    match MonitoringApp::new(args) {
                        Ok(mut monitoring_app) => {
                            elements_container.hide();

                            let _ = monitoring_app.run_client();
                            let webview = WebView::new();
                            elements_container.pack_start(&webview, true, true, 0);

                            let add_camera_btn = Button::with_label("Add camera");
                            elements_container.pack_start(&add_camera_btn, false, false, 0);

                            let add_incident_btn = Button::with_label("Add incident");
                            elements_container.pack_start(&add_incident_btn, false, false, 0);

                            // Load Leaflet map
                            webview.load_uri("https://leafletjs.com/examples/quick-start/index.html");
                            let webview_clone = webview.clone();
                            let monitoring_app_ref = Rc::new(RefCell::new(monitoring_app));
                            let x = Rc::new(RefCell::new(0.0));
                            let y = Rc::new(RefCell::new(0.0));
                            add_camera_btn.connect_clicked(clone!(@strong y, @strong x, @strong monitoring_app_ref => move |_| {
                                let lat = y.borrow().to_string();
                                let long = x.borrow().to_string();
                                let camera_location = Location::new(lat, long);
                                monitoring_app_ref.borrow_mut().add_camera(camera_location);
                            }));
                            add_incident_btn.connect_clicked(clone!(@strong y, @strong x, @strong monitoring_app_ref => move |_| {
                                let lat = y.borrow().to_string();
                                let long = x.borrow().to_string();
                                let incident_location = Location::new(lat, long);
                                monitoring_app_ref.borrow_mut().add_incident(incident_location);
                            }));
                            webview.connect_button_press_event(clone!(@strong x, @strong y, @strong monitoring_app_ref => move |_, event| {
                                if event.button() == 1 {
                                    let pos_x = event.position().0;
                                    let pos_y = event.position().1;
                                    *x.borrow_mut() = pos_x;
                                    *y.borrow_mut() = pos_y;
                                    webview_clone.run_javascript(
                                        &format!("window.map.setView(window.map.containerPointToLatLng([{}, {}]), window.map.getZoom());", pos_x, pos_y),
                                        None::<&gio::Cancellable>,
                                        |_result| (),
                                    );
                                }
                                false.into()
                            }));
                            elements_container.show_all();
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
