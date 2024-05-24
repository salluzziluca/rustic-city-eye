extern crate gtk;

use gtk::glib::clone;
use gtk::{gdk, prelude::*};
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

                handle_connection(&elements_container);

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

///El usuario clickea sobre el mapa, y se guarda la localizacion de ese click.
fn on_webview_button_press_event(x: Rc<RefCell<f64>>, y: Rc<RefCell<f64>>, webview_clone: WebView, event: &gdk::EventButton) -> bool {
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
}

///Se toma la localizacion del click actual, y se crea una camara nueva dentro de la app de monitoreo(en realidad,
/// monitoreo le pasa el Location a su camera system y este es el encargado de crear la camara).
fn on_add_camera_clicked(y: Rc<RefCell<f64>>, x: Rc<RefCell<f64>>, monitoring_app_ref: Rc<RefCell<MonitoringApp>>) {
    let lat = y.borrow().to_string();
    let long = x.borrow().to_string();
    let camera_location = Location::new(lat, long);
    monitoring_app_ref.borrow_mut().add_camera(camera_location);
}

///Se toma la localizacion del click actual, y se crea una incidente nuevo dentro de la app de monitoreo
fn on_add_incident_clicked(y: Rc<RefCell<f64>>, x: Rc<RefCell<f64>>, monitoring_app_ref: Rc<RefCell<MonitoringApp>>) {
    let lat = y.borrow().to_string();
    let long = x.borrow().to_string();
    let incident_location = Location::new(lat, long);
    monitoring_app_ref.borrow_mut().add_incident(incident_location);
}

fn get_connect_form() -> (gtk::Box, Entry, Entry, Entry, Entry, gtk::Button) {
    let vbox = Box::new(Orientation::Vertical, 5);

    let label = Label::new(Some("Ingrese al servidor"));
    let host = Entry::new();
    host.set_placeholder_text(Some("Host: "));

    let port = Entry::new();
    port.set_placeholder_text(Some("Port: "));
    let user = Entry::new();
    user.set_placeholder_text(Some("Username: "));
    let password = Entry::new();
    password.set_placeholder_text(Some("Password: "));

    let connect_btn = Button::with_label("Conectarse");
    vbox.pack_start(&label, false, false, 0);
    vbox.pack_start(&host, false, false, 0);
    vbox.pack_start(&port, false, false, 0);
    vbox.pack_start(&user, false, false, 0);
    vbox.pack_start(&password, false, false, 0);
    vbox.pack_start(&connect_btn, false, false, 0);

    (vbox, host, port, user, password, connect_btn)
}

fn handle_connection(elements_container: &gtk::Box) {
    let (connect_form, host, port, user, password, connect_btn) = get_connect_form();

    elements_container.pack_start(&connect_form, false, false, 0);

    connect_btn.connect_clicked(clone!(@weak host, @weak port, @weak user, @weak password, @weak elements_container => move |_| {
        let args = vec![host.text().to_string(), port.text().to_string(), user.text().to_string(), password.text().to_string()];

        match MonitoringApp::new(args) {
            Ok(mut monitoring_app) => {
                elements_container.foreach(|widget| {
                    elements_container.remove(widget);
                });

                let _ = monitoring_app.run_client();
                let webview = WebView::new();
                elements_container.pack_start(&webview, true, true, 0);

                let add_camera_btn = Button::with_label("Add camera");
                elements_container.pack_start(&add_camera_btn, false, false, 0);

                let add_incident_btn = Button::with_label("Add incident");
                elements_container.pack_start(&add_incident_btn, false, false, 0);

                // Load Leaflet map
                //webview.load_uri("https://leafletjs.com/examples/quick-start/index.html");
                let html_content = format!(r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>Map</title>
                        <meta charset="utf-8" />
                        <meta name="viewport" content="width=device-width, initial-scale=1.0">
                        <link rel="stylesheet" href="https://unpkg.com/leaflet/dist/leaflet.css" />
                        <script src="https://unpkg.com/leaflet/dist/leaflet.js"></script>
                    </head>
                    <body>
                        <div id="map" style="width: 100%; height: 100vh;"></div>
                        <script>
                            var lat = -34.615077;
                            var lon = -58.368084;
                            var map = L.map('map').setView([lat, lon], 16);
                            L.tileLayer('https://{{s}}.tile.openstreetmap.org/{{z}}/{{x}}/{{y}}.png', {{
                                attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
                            }}).addTo(map);
                        </script>
                    </body>
                    </html>
                "#);

                webview.load_html(&html_content, None);
                let webview_clone = webview.clone();
                let monitoring_app_ref = Rc::new(RefCell::new(monitoring_app));
                let x = Rc::new(RefCell::new(0.0));
                let y = Rc::new(RefCell::new(0.0));
                
                add_camera_btn.connect_clicked(clone!(@strong y, @strong x, @strong monitoring_app_ref => move |_| {
                    on_add_camera_clicked(y.clone(), x.clone(), monitoring_app_ref.clone())
                }));
                
                add_incident_btn.connect_clicked(clone!(@strong y, @strong x, @strong monitoring_app_ref => move |_| {
                    on_add_incident_clicked(y.clone(), x.clone(), monitoring_app_ref.clone())
                }));

                webview.connect_button_press_event(clone!(@strong x, @strong y, @strong monitoring_app_ref => move |_, event| {
                    on_webview_button_press_event(x.clone(), y.clone(), webview_clone.clone(), event).into()
                }));

                elements_container.show_all();
            },
            Err(_) => {
                println!("Error: intenta conectarte de nuevo");

                handle_connection(&elements_container);
            },
        }
    }));
}