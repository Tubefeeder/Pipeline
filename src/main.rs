/*
 * Copyright 2021 - 2022 Julian Schmidhuber <github@schmiddi.anonaddy.com>
 *
 * This file is part of Tubefeeder.
 *
 * Tubefeeder is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Tubefeeder is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Tubefeeder.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use gdk::{
    prelude::{ApplicationExt, ApplicationExtManual},
    Display,
};
use gdk_pixbuf::{gio::Settings, prelude::SettingsExt};
use gtk::{traits::GtkWindowExt, CssProvider, StyleContext};

mod config;
use self::config::{APP_ID, GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_BYTES};

mod csv_file_manager;
mod downloader;
mod gui;
mod import;
mod player;

fn init_setting(env: &'static str, value: &str) {
    if std::env::var_os(env).is_none() {
        std::env::set_var(env, value);
    }
}

fn init_settings() {
    let settings = Settings::new(APP_ID);
    init_setting("PLAYER", &settings.string("player"));
    init_setting("DOWNLOADER", &settings.string("downloader"));
    init_setting("PIPED_API_URL", &settings.string("piped-url"));
}

fn init_resources() {
    let gbytes = gtk::glib::Bytes::from_static(RESOURCES_BYTES);
    let resource = gtk::gio::Resource::from_data(&gbytes).unwrap();

    gtk::gio::resources_register(&resource);
}

fn init_folders() {
    let mut user_cache_dir = gtk::glib::user_cache_dir();
    user_cache_dir.push("tubefeeder");

    if !user_cache_dir.exists() {
        std::fs::create_dir_all(&user_cache_dir).expect("could not create user cache dir");
    }

    let mut user_data_dir = gtk::glib::user_data_dir();
    user_data_dir.push("tubefeeder");

    if !user_data_dir.exists() {
        std::fs::create_dir_all(user_data_dir.clone()).expect("could not create user data dir");
    }
}

fn init_internationalization() -> Result<(), Box<dyn std::error::Error>> {
    gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR)?;
    gettextrs::textdomain(GETTEXT_PACKAGE)?;
    Ok(())
}

fn init_css() {
    let provider = CssProvider::new();
    provider.load_from_resource("/de/schmidhuberj/tubefeeder/style.css");

    // Add the provider to the default screen
    StyleContext::add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

#[tokio::main]
async fn main() {
    env_logger::init();
    init_internationalization().expect("Failed to initialize internationalization");

    gtk::init().expect("Failed to initialize gtk");
    libadwaita::init().expect("Failed to initialize libadwaita");
    let app = gtk::Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    init_resources();
    init_folders();
    init_settings();
    init_css();
    // Create new window and present it
    let window = crate::gui::window::Window::new(app);
    window.present();
}
