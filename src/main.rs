mod errors;
mod gui;
mod subscriptions;
mod youtube_feed;

use crate::gui::app::App;

use gio::prelude::*;

#[tokio::main]
async fn main() {
    let app = gtk::Application::new(Some("com.github.schmiddiii.tubefeeder"), Default::default())
        .expect("Initialization failed...");
    app.connect_activate(move |app| App::init(app));

    app.run(&std::env::args().collect::<Vec<_>>());
}
