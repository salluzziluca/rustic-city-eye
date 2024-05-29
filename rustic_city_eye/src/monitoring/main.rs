extern crate gtk; // Import the gtk crate for GTK widget functionality
use gtk::{prelude::*, Application, Entry, Label, Overlay}; // Import specific items from gtk module
use gtk::{Button, Box, Image, Window, WindowType, Orientation}; // Import specific items from gtk module
use std::cell::RefCell; // Import RefCell for interior mutability
use std::rc::Rc; // Import Rc for reference counting

use gtk::glib::clone; // Import the clone function from glib module
use gtk::glib; // Import the glib module

// Import modules from the project
use rustic_city_eye::monitoring::monitoring_app::MonitoringApp;
use rustic_city_eye::mqtt::protocol_error::ProtocolError;
use rustic_city_eye::surveilling::location::Location;

fn main() -> Result<(), ProtocolError> {
    // Create a new GTK application
    let app = Application::builder()
        .application_id("com.example.RusticCityEye")
        .build();

    // Connect to the activate signal of the application
    app.connect_activate(|app| {
        // Create the main window
        let home_window = Window::new(WindowType::Toplevel);
        home_window.set_title("Rustic City Eye");
        home_window.set_default_size(2000, 1500);

        // Create a vertical box container
        let vbox = Box::new(Orientation::Vertical, 5);

        // Create a button for connecting to a server
        let button = Button::with_label("Conectarse a un servidor");
        vbox.pack_start(&button, false, false, 0);

        // Create a container for elements
        let elements_container = Box::new(Orientation::Vertical, 5);
        vbox.pack_start(&elements_container, true, true, 0);

        // Connect the button clicked signal
        button.connect_clicked(clone!(@weak button, @weak elements_container => move |_| {
            button.hide();
            handle_connection(&elements_container);
            elements_container.show_all();
        }));

        // Add the vertical box container to the window
        home_window.add(&vbox);

        // Show all widgets in the window
        home_window.show_all();

        // Set the application for the main window
        home_window.set_application(Some(app));
    });

    // Run the GTK application
    app.run();

    Ok(())
}

// Function to handle connection
fn handle_connection(elements_container: &gtk::Box) {
    let (connect_form, host, port, user, password, connect_btn) = get_connect_form();

    // Pack the connection form into the elements container
    elements_container.pack_start(&connect_form, false, false, 0);

    // Connect the connect button clicked signal
    connect_btn.connect_clicked(clone!(@weak host, @weak port, @weak user, @weak password, @weak elements_container => move |_| {
        let args = vec![host.text().to_string(), port.text().to_string(), user.text().to_string(), password.text().to_string()];

        match MonitoringApp::new(args) {
            Ok(monitoring_app) => {
                let monitoring_app_ref = Rc::new(RefCell::new(monitoring_app));
                handle_map(&elements_container, monitoring_app_ref)
            },
            Err(_) => {
                println!("Error: intenta conectarte de nuevo");
                handle_connection(&elements_container);
            },
        }
    }));
}

// Function to handle map display
fn handle_map(elements_container: &gtk::Box,  monitoring_app_ref: Rc<RefCell<MonitoringApp>>) {
    // Remove all existing widgets from the elements container
    elements_container.foreach(|widget| {
        elements_container.remove(widget);
    });

    // Run the client for monitoring
    let _ = monitoring_app_ref.borrow_mut().run_client();

    // Create a horizontal box to hold the map and panel
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);

    // Create a new image widget for the map
    let image = gtk::Image::from_file("src/monitoring/Map.png");

    // Create a new event box to hold the image
    let event_box = gtk::EventBox::new();
    event_box.add(&image);

    // Create an overlay to overlay the image with other widgets
    let overlay = Overlay::new();
    overlay.add(&event_box);
    hbox.add(&overlay);
    
    // Initialize coordinates for click event
    let (x, y) = (0.0, 0.0);
    let x_ref = Rc::new(RefCell::new(x));
    let y_ref = Rc::new(RefCell::new(y));

    let overlay_rc = Rc::new(RefCell::new(overlay));
    // Connect button press event for the event box
    event_box.connect_button_press_event({
        let x_ref = Rc::clone(&x_ref);
        let y_ref = Rc::clone(&y_ref);
        move |_, event| {
            x_ref.replace(event.position().0);
            y_ref.replace(event.position().1);
            println!("Image clicked at coordinates {},{}", x_ref.borrow(), y_ref.borrow());
            false.into()
        }
    });

    // Create a vertical box for buttons
    let vbox = Box::new(Orientation::Vertical, 0);

    // Create a button for adding a camera
    let add_camera_button = Button::with_label("Add Camera");
    vbox.add(&add_camera_button);

    // Create a button for adding an incident
    let add_incident_button = Button::with_label("Add Incident");
    vbox.add(&add_incident_button);
    hbox.add(&vbox);

    // Create a box for camera list
    let camera_list_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
    vbox.pack_start(&camera_list_box, true, true, 0);
    let camera_list_box = Rc::new(RefCell::new(camera_list_box));

    // Create a box for incident list
    let incident_list_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
    vbox.pack_start(&incident_list_box, true, true, 0);
    let incident_list_box = Rc::new(RefCell::new(incident_list_box));

    // Connect button signals for adding camera and incident
    add_camera_button.connect_clicked({
        let x_ref = Rc::clone(&x_ref);
        let y_ref = Rc::clone(&y_ref);
        let monitoring_app_ref = Rc::clone(&monitoring_app_ref);
        let camera_list_box = Rc::clone(&camera_list_box);
        move |_| {
            on_add_camera_clicked(RefCell::clone(&y_ref), RefCell::clone(&x_ref), Rc::clone(&camera_list_box), Rc::clone(&monitoring_app_ref), Rc::clone(&overlay_rc));
        }
    });

    add_incident_button.connect_clicked({
        let x_ref = Rc::clone(&x_ref);
        let y_ref = Rc::clone(&y_ref);
        let monitoring_app_ref = Rc::clone(&monitoring_app_ref);
        let incident_list_box = Rc::clone(&incident_list_box);
        move |_| {
            on_add_incident_clicked(Rc::clone(&y_ref), Rc::clone(&x_ref), &Rc::clone(&monitoring_app_ref), Rc::clone(&incident_list_box));
        }
   
    });

    // Pack the hbox into the elements container
    elements_container.pack_start(&hbox, true, true, 0);
    // Show all widgets in the elements container
    elements_container.show_all();
}

// Function to handle adding a camera
fn on_add_camera_clicked(
    y: RefCell<f64>,
    x: RefCell<f64>,
    camera_list_box: Rc<RefCell<gtk::Box>>,
    monitoring_app_ref: Rc<RefCell<MonitoringApp>>,
    overlay: Rc<RefCell<gtk::Overlay>>,
) {
    // Extract latitude and longitude from coordinates
    let lat = format!("{:.2}", y.borrow());
    let long = format!("{:.2}", x.borrow());
    let camera_location = Location::new(lat, long);
    monitoring_app_ref.borrow_mut().add_camera(camera_location);

    // Clear the camera list panel
    camera_list_box.borrow().foreach(|widget| {
        camera_list_box.borrow().remove(widget);
    });

    // Fetch the list of cameras from the monitoring app
    let binding = monitoring_app_ref.borrow();
    let cameras = binding.get_cameras();

    let camera_list_box: gtk::Box = camera_list_box.borrow().clone().downcast().unwrap();
    // Add each camera to the list box
    for camera in cameras {
        let location = camera.get_location();
        let camera_label = gtk::Label::new(Some(&format!(
            "Camera at ({}, {})",
            location.latitude, location.longitude
        )));
        camera_list_box.pack_start(&camera_label, false, false, 0);

        let camera_image = Image::from_file("src/monitoring/Camera.png");

        // Add camera image to overlay
        let mut overlay_ref_mut = overlay.borrow_mut();
        let overlay = &mut *overlay_ref_mut;
        overlay.add_overlay(&camera_image);

        // Set alignment for overlay
        camera_image.set_halign(gtk::Align::Start);
        camera_image.set_valign(gtk::Align::Start);

        // Calculate position for camera image
        let image_size_offset = 120;
        let x_pos = location.longitude as i32 - image_size_offset;
        let y_pos = location.latitude as i32 - image_size_offset;

        camera_image.set_margin_start(x_pos);
        camera_image.set_margin_top(y_pos);

        // Get the original size of the image
        let (original_width, original_height) = camera_image.pixbuf().map_or((0, 0), |pixbuf| {
            (pixbuf.width(), pixbuf.height())
        });

        // Calculate new dimensions for scaling
        let scale_factor: f32 = 0.4; // Change this to adjust the scale
        let new_width = (original_width as f32 * scale_factor) as i32;
        let new_height = (original_height as f32 * scale_factor) as i32;

        // Set the desired size for the image
        camera_image.set_size_request(new_width, new_height);

        camera_image.show(); // Ensure the image is visible
    }
    camera_list_box.show_all();
}

/// Function to handle adding an incident
fn on_add_incident_clicked(
    y: Rc<RefCell<f64>>,
    x: Rc<RefCell<f64>>,
    monitoring_app_ref: &Rc<RefCell<MonitoringApp>>,
    incident_list_box: Rc<RefCell<gtk::Box>>,
) {
    // Extract latitude and longitude from coordinates
    let lat = y.borrow().to_string();
    let long = x.borrow().to_string();
    let incident_location = Location::new(lat, long);
    monitoring_app_ref
        .borrow_mut()
        .add_incident(incident_location);
    // Update the incident list
    update_incident_list(&incident_list_box.borrow(), monitoring_app_ref);
}

// Function to update incident list
fn update_incident_list(incident_list_box: &gtk::Box, monitoring_app_ref: &Rc<RefCell<MonitoringApp>>) {
    // Clear the existing incident list
    incident_list_box.foreach(|widget| {
        incident_list_box.remove(widget);
    });

    // Fetch the list of incidents from the monitoring app
    let incidents = monitoring_app_ref.borrow().get_incidents();

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

// Function to create connection form elements
fn get_connect_form() -> (gtk::Box, Entry, Entry, Entry, Entry, gtk::Button) {
    let vbox = Box::new(Orientation::Vertical, 5);

    // Create labels and entries for host, port, username, password
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

    // Pack labels, entries, and button into vertical box
    vbox.pack_start(&label, false, false, 0);
    vbox.pack_start(&host, false, false, 0);
    vbox.pack_start(&port, false, false, 0);
    vbox.pack_start(&user, false, false, 0);
    vbox.pack_start(&password, false, false, 0);
    vbox.pack_start(&connect_btn, false, false, 0);

    (vbox, host, port, user, password, connect_btn)
}

