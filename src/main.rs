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

mod csv_file_manager;
mod gui;
mod player;

use relm::RelmApp;

use crate::gui::AppModel;

fn init_resource() {
    let res_bytes = include_bytes!("../resources.gresource");

    let gbytes = glib::Bytes::from_static(res_bytes.as_ref());
    let resource = gio::Resource::from_data(&gbytes).unwrap();

    gio::resources_register(&resource);
}
#[tokio::main]
async fn main() {
    env_logger::init();
    let _ = gtk::init();
    let _ = libadwaita::init();
    init_resource();

    let joiner = tf_join::Joiner::new();

    let model = AppModel::new(joiner);
    let app = RelmApp::new(model);
    app.run()
}
