/*
 * Copyright 2021 Julian Schmidhuber <github@schmiddi.anonaddy.com>
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

use gdk::prelude::{ApplicationExt, ApplicationExtManual};
use gtk::glib::IsA;
use gtk::traits::{GtkWindowExt, WidgetExt};

mod csv_file_manager;
mod downloader;
mod gui;
mod player;

fn init_resources() {
    let res_bytes = include_bytes!("../resources.gresource");

    let gbytes = gtk::glib::Bytes::from_static(res_bytes.as_ref());
    let resource = gtk::gio::Resource::from_data(&gbytes).unwrap();

    gtk::gio::resources_register(&resource);
}

fn init_icons<P: IsA<gdk::Display>>(display: &P) {
    let icon_theme = gtk::IconTheme::for_display(display);

    icon_theme.add_resource_path("/");
    icon_theme.add_resource_path("/org/gnome/design/IconLibrary/data/icons/");
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

#[tokio::main]
async fn main() {
    env_logger::init();
    gtk::init().expect("Failed to initialize gtk");
    libadwaita::init();
    let app = gtk::Application::builder()
        .application_id("de.schmidhuberj.tubefeeder")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    init_resources();
    init_folders();
    // Create new window and present it
    let window = crate::gui::window::Window::new(app);
    init_icons(&window.display());
    window.present();
}
