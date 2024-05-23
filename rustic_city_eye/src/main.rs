use gtk::glib::clone;
use gtk::{glib, prelude::*, Window, WindowType};
use gtk::{Application, Label, Box, Entry, Orientation, Button};
use rustic_city_eye::monitoring::monitoring_app::MonitoringApp;
use rustic_city_eye::mqtt::protocol_error::ProtocolError;

fn main() -> Result<(), ProtocolError> {
    let app = Application::builder()
        .application_id("com.example.RusticCityEye")
        .build();

        app.connect_activate(|app| {
            let home_window = Window::new(WindowType::Toplevel);
            home_window.set_title("Rustic City Eye");
            home_window.set_default_size(300, 200);

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

                connect_btn.connect_clicked(clone!(@weak host, @weak port, @weak user, @weak password => move |_| {
                    let h = host.text().to_string();
                    let po = port.text().to_string();
                    let u = user.text();
                    let p = password.text();
                    let mut args = Vec::new();
                    args.push(h);
                    args.push(po);
                    println!("user {} password {}", u, p);

                    let _ = match MonitoringApp::new(args) {
                        Ok(mut monitoring_app) => monitoring_app.app_run(),
                        Err(_) => todo!(),
                    };
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