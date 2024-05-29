// extern crate gtk;

// use gtk::glib::clone;
// use gtk::{gdk, prelude::*};
// use gtk::{glib, Window, WindowType};
// use gtk::{Application, Box, Button, Entry, Label, Orientation};
// use webkit2gtk::gio;
// use webkit2gtk::{WebView, WebViewExt};

// use std::cell::RefCell;
// use std::rc::Rc;

// use rustic_city_eye::monitoring::monitoring_app::MonitoringApp;
// use rustic_city_eye::mqtt::protocol_error::ProtocolError;
// use rustic_city_eye::surveilling::location::Location;

// fn main() -> Result<(), ProtocolError> {
//     let app = Application::builder()
//         .application_id("com.example.RusticCityEye")
//         .build();

//     app.connect_activate(|app| {
//         let home_window = Window::new(WindowType::Toplevel);
//         home_window.set_title("Rustic City Eye");
//         home_window.set_default_size(2000, 1500);

//         let vbox = Box::new(Orientation::Vertical, 5);

//         let button = Button::with_label("Conectarse a un servidor");
//         vbox.pack_start(&button, false, false, 0);

//         let elements_container = Box::new(Orientation::Vertical, 5);
//         vbox.pack_start(&elements_container, true, true, 0);

//         button.connect_clicked(clone!(@weak button, @weak elements_container => move |_| {
//             button.hide();

//             handle_connection(&elements_container);

//             elements_container.show_all();
//         }));

//         // Agrega la caja a la ventana.
//         home_window.add(&vbox);

//         // Muestra todos los widgets en la ventana.
//         home_window.show_all();

//         // Configura la aplicación principal.
//         home_window.set_application(Some(app));
//     });

//     app.run();

//     Ok(())
// }

// #[allow(clippy::useless_format)]
// fn handle_connection(elements_container: &gtk::Box) {
//     let (connect_form, host, port, user, password, connect_btn) = get_connect_form();

//     elements_container.pack_start(&connect_form, false, false, 0);

//     connect_btn.connect_clicked(clone!(@weak host, @weak port, @weak user, @weak password, @weak elements_container => move |_| {
//         let args = vec![host.text().to_string(), port.text().to_string(), user.text().to_string(), password.text().to_string()];

//         match MonitoringApp::new(args) {
//             Ok(mut monitoring_app) => {
//                 elements_container.foreach(|widget| {
//                     elements_container.remove(widget);
//                 });

//                 let _ = monitoring_app.run_client();

//                 // Create the horizontal box to hold the map and the panel
//                 let hbox = Box::new(Orientation::Horizontal, 5);
//                 elements_container.pack_start(&hbox, true, true, 0);

//                 let webview = WebView::new();
//                 hbox.pack_start(&webview, true, true, 0);

//                 // Create the panel for camera list and buttons
//                 let panel = Box::new(Orientation::Vertical, 5);
//                 hbox.pack_start(&panel, false, false, 0);

//                 let add_camera_btn = Button::with_label("Add camera");
//                 panel.pack_start(&add_camera_btn, false, false, 0);

//                 let add_incident_btn = Button::with_label("Add incident");
//                 panel.pack_start(&add_incident_btn, false, false, 0);

//                 // Create the camera list panel
//                 let camera_list_box = Box::new(Orientation::Vertical, 5);
//                 panel.pack_start(&camera_list_box, true, true, 0);

//                 // Load Leaflet map
//                 let html_content = format!(r#"
//                     <!DOCTYPE html>
//                     <html>
//                     <head>
//                         <title>Map</title>
//                         <meta charset="utf-8" />
//                         <meta name="viewport" content="width=device-width, initial-scale=1.0">
//                         <link rel="stylesheet" href="https://unpkg.com/leaflet/dist/leaflet.css" />
//                         <script src="https://unpkg.com/leaflet/dist/leaflet.js"></script>
//                     </head>
//                     <body>
//                         <div id="map" style="width: 100%; height: 100vh;"></div>
//                         <script>
//                             var lat = -34.615077;
//                             var lon = -58.368084;
//                             var map = L.map('map').setView([lat, lon], 16);
//                             L.tileLayer('https://{{s}}.tile.openstreetmap.org/{{z}}/{{x}}/{{y}}.png', {{
//                                     attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
//                                 }}).addTo(map);
//                                 function getLatLngFromPixel(x, y) {{
//                                         var latlng = map.containerPointToLatLng([x, y]);
//                                         return {{ lat: latlng.lat, lng: latlng.lng }};
//                                     }}
//                         </script>
//                     </body>
//                     </html>
//                 "#);

//                 webview.load_html(&html_content, None);
//                 let webview_clone = webview.clone();
//                 let monitoring_app_ref = Rc::new(RefCell::new(monitoring_app));
//                 let x = Rc::new(RefCell::new(0.0));
//                 let y = Rc::new(RefCell::new(0.0));
//                 add_camera_btn.connect_clicked(clone!(@strong y, @strong x, @strong monitoring_app_ref, @strong camera_list_box => move |_| {
//                     on_add_camera_clicked(y.clone(), x.clone(), monitoring_app_ref.clone(), Rc::new(RefCell::new(camera_list_box.clone())));
//                 }));
//                 add_incident_btn.connect_clicked(clone!(@strong y, @strong x, @strong monitoring_app_ref => move |_| {
//                     on_add_incident_clicked(y.clone(), x.clone(), monitoring_app_ref.clone());
//                 }));

//                 webview.connect_button_press_event(clone!(@strong x, @strong y, @strong monitoring_app_ref => move |_, event| {
//                     on_webview_button_press_event(x.clone(), y.clone(), webview_clone.clone(), event).into()
//                 }));

//                 elements_container.show_all();

//                 // Initial call to populate the camera list
//                 update_camera_list(&camera_list_box, &monitoring_app_ref);
//             },
//             Err(_) => {
//                 println!("Error: intenta conectarte de nuevo");

//                 handle_connection(&elements_container);
//             },
//         }
//     }));
// }

// fn get_connect_form() -> (gtk::Box, Entry, Entry, Entry, Entry, gtk::Button) {
//     let vbox = Box::new(Orientation::Vertical, 5);

//     let label = Label::new(Some("Ingrese al servidor"));

//     let host = Entry::new();
//     host.set_placeholder_text(Some("Host: "));

//     let port = Entry::new();
//     port.set_placeholder_text(Some("Port: "));

//     let user = Entry::new();
//     user.set_placeholder_text(Some("Username: "));

//     let password = Entry::new();
//     password.set_placeholder_text(Some("Password: "));

//     let connect_btn = Button::with_label("Conectarse");
//     vbox.pack_start(&label, false, false, 0);
//     vbox.pack_start(&host, false, false, 0);
//     vbox.pack_start(&port, false, false, 0);
//     vbox.pack_start(&user, false, false, 0);
//     vbox.pack_start(&password, false, false, 0);
//     vbox.pack_start(&connect_btn, false, false, 0);

//     (vbox, host, port, user, password, connect_btn)
// }

// fn update_camera_list(camera_list_box: &gtk::Box, monitoring_app_ref: &Rc<RefCell<MonitoringApp>>) {
//     // Clear the existing list
//     camera_list_box.foreach(|widget| {
//         camera_list_box.remove(widget);
//     });

//     // Fetch the list of cameras from the monitoring app
//     let cameras = monitoring_app_ref.borrow().get_cameras(); // Assuming you have a method to get the list of cameras

//     // Add each camera to the list box
//     for camera in cameras {
//         let location = camera.get_location();
//         let camera_label = Label::new(Some(&format!(
//             "Camera at ({}, {})",
//             location.latitude, location.longitude
//         )));
//         camera_list_box.pack_start(&camera_label, false, false, 0);
//     }

//     camera_list_box.show_all();
// }

// fn on_add_camera_clicked(
//     y: Rc<RefCell<f64>>,
//     x: Rc<RefCell<f64>>,
//     monitoring_app_ref: Rc<RefCell<MonitoringApp>>,
//     camera_list_box: Rc<RefCell<gtk::Box>>,
// ) {
//     let lat = format!("{:.2}", y.borrow());
//     let long = format!("{:.2}", x.borrow());
//     let camera_location = Location::new(lat, long);
//     monitoring_app_ref.borrow_mut().add_camera(camera_location);

//     // Update the camera list panel
//     update_camera_list(&camera_list_box.borrow(), &monitoring_app_ref);
// }

// fn on_webview_button_press_event(
//     x: Rc<RefCell<f64>>,
//     y: Rc<RefCell<f64>>,
//     webview_clone: WebView,
//     event: &gdk::EventButton,
// ) -> bool {
//     if event.button() == 1 {
//         let pos_x = event.position().0;
//         let pos_y = event.position().1;

//         let _script = format!(
//             "var result = getLatLngFromPixel({}, {});
//              JSON.stringify(result);",
//             pos_x, pos_y
//         );

//         *x.borrow_mut() = pos_x;
//         *y.borrow_mut() = pos_y;

//         webview_clone.run_javascript(
//             &format!("window.map.setView(window.map.containerPointToLatLng([{}, {}]), window.map.getZoom());", pos_x, pos_y),
//             None::<&gio::Cancellable>,
//             |_result| (),
//         );
//     }
//     false
// }

// ///Se toma la localizacion del click actual, y se crea una incidente nuevo dentro de la app de monitoreo
// fn on_add_incident_clicked(
//     y: Rc<RefCell<f64>>,
//     x: Rc<RefCell<f64>>,
//     monitoring_app_ref: Rc<RefCell<MonitoringApp>>,
// ) {
//     let lat = y.borrow().to_string();
//     let long = x.borrow().to_string();
//     let incident_location = Location::new(lat, long);
//     monitoring_app_ref
//         .borrow_mut()
//         .add_incident(incident_location);
// }

extern crate gtk;

use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{glib, Window, WindowType};
use gtk::{Application, Box, Button, Entry, Label, Orientation};

use std::cell::RefCell;
use std::rc::Rc;

use rustic_city_eye::monitoring::monitoring_app::MonitoringApp;
use rustic_city_eye::mqtt::protocol_error::ProtocolError;
use rustic_city_eye::surveilling::location::Location;

#[cfg(not(test))]
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

                // Create the horizontal box to hold the map and the panel
                let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);
                elements_container.pack_start(&hbox, true, true, 0);

                // Create a new image widget
                let image = gtk::Image::from_file("src/monitoring/Map.png");

                // Create a new event box
                let event_box = gtk::EventBox::new();
                event_box.add(&image);

                // Add the event box to the horizontal box
                hbox.add(&event_box);

                // Create a vertical box to hold buttons for the panel
                let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

                // Create a button for adding a camera
                let add_camera_button = gtk::Button::with_label("Add Camera");
                vbox.add(&add_camera_button);

                // Create a button for adding an incident
                let add_incident_button = gtk::Button::with_label("Add Incident");
                vbox.add(&add_incident_button);

                // Create the panel for camera list and buttons
                let panel = gtk::Box::new(gtk::Orientation::Vertical, 5);
                hbox.pack_start(&panel, false, false, 0);

                let add_camera_btn = gtk::Button::with_label("Add camera");
                panel.pack_start(&add_camera_btn, false, false, 0);

                let add_incident_btn = gtk::Button::with_label("Add incident");
                panel.pack_start(&add_incident_btn, false, false, 0);

                // Create the camera list panel
                let camera_list_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
                panel.pack_start(&camera_list_box, true, true, 0);
                let camera_list_box = Rc::new(RefCell::new(camera_list_box));

                let incident_list_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
                panel.pack_start(&incident_list_box, true, true, 0);
                let incident_list_box = Rc::new(RefCell::new(incident_list_box));

                let monitoring_app_ref = Rc::new(RefCell::new(monitoring_app));

                let (x, y) = (0.0, 0.0);
                let x_ref = Rc::new(RefCell::new(x));
                let y_ref = Rc::new(RefCell::new(y));
                event_box.connect_button_press_event({
                    let x_ref = Rc::clone(&x_ref);
                    let y_ref = Rc::clone(&y_ref);
                    move |_, event| {
                        x_ref.replace(event.position().0);
                        y_ref.replace(event.position().1);
                        println!("Image clicked at coordinates {},{}", x_ref.borrow(), y_ref.borrow());
                        false.into() // Ensure you return the correct type here
                    }
                });

                add_camera_btn.connect_clicked({
                    let x_ref = Rc::clone(&x_ref);
                    let y_ref = Rc::clone(&y_ref);
                    let monitoring_app_ref = Rc::clone(&monitoring_app_ref);
                    let camera_list_box = Rc::clone(&camera_list_box);
                    move |_| {
                        on_add_camera_clicked(Rc::clone(&y_ref), Rc::clone(&x_ref), Rc::clone(&monitoring_app_ref), Rc::clone(&camera_list_box));
                    }
                });

                add_incident_btn.connect_clicked({
                    let x_ref = Rc::clone(&x_ref);
                    let y_ref = Rc::clone(&y_ref);
                    let monitoring_app_ref = Rc::clone(&monitoring_app_ref);
                    let incident_list_box = Rc::clone(&incident_list_box);
                    move |_| {
                        on_add_incident_clicked(Rc::clone(&y_ref), Rc::clone(&x_ref), Rc::clone(&monitoring_app_ref), Rc::clone(&incident_list_box));
                    }
                });


                // add_incident_btn.connect_clicked({
                //     let x_ref = Rc::clone(&x_ref);
                //     let y_ref = Rc::clone(&y_ref);
                //     let monitoring_app_ref = Rc::clone(&monitoring_app_ref);
                //     move |_| {
                //         on_add_incident_clicked(Rc::clone(&y_ref), Rc::clone(&x_ref), Rc::clone(&monitoring_app_ref));
                //     }
                // });

                elements_container.add(&hbox);

                elements_container.show_all();

                // Initial call to populate the camera list
                update_camera_list(&camera_list_box.borrow(), &monitoring_app_ref);
            },
            Err(_) => {
                println!("Error: intenta conectarte de nuevo");

                handle_connection(&elements_container);
            },
        }
        }));
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

fn update_camera_list(camera_list_box: &gtk::Box, monitoring_app_ref: &Rc<RefCell<MonitoringApp>>) {
    // Clear the existing list
    camera_list_box.foreach(|widget| {
        camera_list_box.remove(widget);
    });

    // Fetch the list of cameras from the monitoring app
    let cameras = monitoring_app_ref.borrow().get_cameras(); // Assuming you have a method to get the list of cameras

    // Add each camera to the list box
    for camera in cameras {
        let location = camera.get_location();
        let camera_label = gtk::Label::new(Some(&format!(
            "Camera at ({}, {})",
            location.latitude, location.longitude
        )));
        camera_list_box.pack_start(&camera_label, false, false, 0);
    }

    camera_list_box.show_all();
}

fn on_add_camera_clicked(
    y: Rc<RefCell<f64>>,
    x: Rc<RefCell<f64>>,
    monitoring_app_ref: Rc<RefCell<MonitoringApp>>,
    camera_list_box: Rc<RefCell<gtk::Box>>,
) {
    let lat = format!("{:.2}", y.borrow());
    let long = format!("{:.2}", x.borrow());
    let camera_location = Location::new(lat, long);
    monitoring_app_ref.borrow_mut().add_camera(camera_location);

    // Update the camera list panel
    update_camera_list(&camera_list_box.borrow(), &monitoring_app_ref);
}

///Se toma la localizacion del click actual, y se crea una incidente nuevo dentro de la app de monitoreo
fn on_add_incident_clicked(
    y: Rc<RefCell<f64>>,
    x: Rc<RefCell<f64>>,
    monitoring_app_ref: Rc<RefCell<MonitoringApp>>,
    incident_list_box: Rc<RefCell<gtk::Box>>,
) {
    let lat = y.borrow().to_string();
    let long = x.borrow().to_string();
    let incident_location = Location::new(lat, long);
    monitoring_app_ref
        .borrow_mut()
        .add_incident(incident_location);
    update_incident_list(&incident_list_box.borrow(), &monitoring_app_ref);
}

fn update_incident_list(
    incident_list_box: &gtk::Box,
    monitoring_app_ref: &Rc<RefCell<MonitoringApp>>,
) {
    // Clear the existing list
    incident_list_box.foreach(|widget| {
        incident_list_box.remove(widget);
    });

    // Fetch the list of incidents from the monitoring app
    let incidents = monitoring_app_ref.borrow().get_incidents(); // Assuming you have a method to get the list of incidents

    // Add each incident to the list box
    for incident in incidents {
        let location = incident.get_location();
        let incident_label = gtk::Label::new(Some(&format!(
            "Incident at ({:.2}, {:.2})",
            location.latitude, location.longitude
        )));
        incident_list_box.pack_start(&incident_label, false, false, 0);
    }

    incident_list_box.show_all();
}
