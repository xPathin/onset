use gtk4::gio;
use gtk4::prelude::*;
use libadwaita as adw;

use crate::ui::MainWindow;

const APP_ID: &str = "com.github.xPathin.onset";

pub fn run() -> i32 {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::default())
        .build();

    app.connect_startup(|_| {
        adw::init().expect("Failed to initialize libadwaita");
    });

    app.connect_activate(build_ui);

    app.run().into()
}

fn build_ui(app: &adw::Application) {
    let window = MainWindow::build(app);
    window.present();
}
