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

use std::thread;

use gdk_pixbuf::{Colorspace, Pixbuf};
use gtk::prelude::*;
use relm::{Channel, Relm, Widget};
use relm_derive::{widget, Msg};
use tf_core::Video;
use tf_join::AnyVideo;

const WIDTH: i32 = 120;
const HEIGHT: i32 = 90;

pub fn default_pixbuf() -> Pixbuf {
    let pixbuf =
        Pixbuf::new(Colorspace::Rgb, true, 8, WIDTH, HEIGHT).expect("Could not create empty");
    pixbuf.fill(0);

    pixbuf
}

pub struct ThumbnailModel {
    relm: Relm<Thumbnail>,
    video: AnyVideo,
}

#[derive(Msg)]
pub enum ThumbnailMsg {
    SetImage,
    SetImagePixbuf(Pixbuf),
}

#[widget]
impl Widget for Thumbnail {
    fn model(relm: &Relm<Self>, video: AnyVideo) -> ThumbnailModel {
        ThumbnailModel {
            relm: relm.clone(),
            video,
        }
    }

    fn update(&mut self, event: ThumbnailMsg) {
        match event {
            ThumbnailMsg::SetImage => self.set_image(),
            ThumbnailMsg::SetImagePixbuf(pixbuf) => {
                self.set_image_pixbuf(pixbuf);
            }
        }
    }

    fn set_image(&mut self) {
        let stream = self.model.relm.stream().clone();

        let (_channel, sender) = Channel::new(move |path| {
            stream.emit(ThumbnailMsg::SetImagePixbuf(
                Pixbuf::from_file(path).unwrap_or_else(|_| default_pixbuf()),
            ));
        });

        let video = self.model.video.clone();
        thread::spawn(move || {
            let mut user_data_dir = glib::user_cache_dir();
            user_data_dir.push("tubefeeder");
            user_data_dir.push(&format!("{}.png", video.title()));
            let path = user_data_dir;
            // TODO: Use same reqwest::Client for all thumbnail queries.
            // TODO: Use Caching
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { video.thumbnail(path.clone(), WIDTH, HEIGHT).await });
            sender.send(path).expect("Could not send pixbuf");
        });
    }

    fn set_image_pixbuf(&mut self, pixbuf: Pixbuf) {
        self.widgets.image.set_from_pixbuf(Some(&pixbuf));
    }

    fn init_view(&mut self) {
        self.widgets.image.set_from_pixbuf(Some(&default_pixbuf()));
    }

    view! {
        gtk::Box {
            #[name="image"]
            gtk::Image {}
        },
    }
}
