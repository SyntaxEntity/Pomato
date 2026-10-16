mod app;
mod config;
mod notifier;
mod timer;
mod tray;

use relm4::RelmApp;

const CSS: &str = include_str!("../data/style.css");

fn main() {
    adw::init().expect("Failed to initialize libadwaita");

    let provider = gtk::CssProvider::new();
    provider.load_from_string(CSS);
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let app = RelmApp::new("io.github.pomato.Pomato");
    app.run::<app::App>(());
}
