use gtk::{glib, prelude::*};
use gtk::{Application, ApplicationWindow, Label, Box, Entry, Orientation, Button};

fn main() {
    let app = Application::builder()
        .application_id("org.example.HelloWorld")
        .build();

    app.connect_activate(|app| {
        // We create the main window.
        let win = ApplicationWindow::builder()
            .application(app)
            .default_width(320)
            .default_height(200)
            .title("Rustic City Eye")
            .build();
        let vbox = Box::new(Orientation::Vertical, 5);
        let label = Label::new(Some("Hello, GTK!"));
        let entry = Entry::new();
        let button = Button::with_label("Submit");

        // Connect the button's "clicked" signal to a callback function
        button.connect_clicked(glib::clone!(@weak entry => move |_| {
            // Get the text from the entry widget
            let text = entry.text();
            // Print the text to the console
            println!("Entry text: {}", text);
        }));

        // Add the label and entry to the box
        vbox.pack_start(&label, false, false, 0);
        vbox.pack_start(&entry, false, false, 0);
        vbox.pack_start(&button, false, false, 0);
        // Don't forget to make all widgets visible.
        win.add(&vbox);
        win.show_all();
    });

    app.run();
}