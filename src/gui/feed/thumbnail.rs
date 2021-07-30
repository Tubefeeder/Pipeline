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
 * along with Foobar.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::youtube_feed;

use std::fs::File;
use std::io::{Read, Write};
use std::thread;

use bytes::Bytes;
use gdk_pixbuf::{Colorspace, Pixbuf};
use gio::{MemoryInputStream, NONE_CANCELLABLE};
use gtk::prelude::*;
use relm::{Channel, Relm, Widget};
use relm_derive::{widget, Msg};

const WIDTH: i32 = 120;
const HEIGHT: i32 = 90;

pub struct ThumbnailModel {
    url: String,
    relm: Relm<Thumbnail>,
}

#[derive(Msg)]
pub enum ThumbnailMsg {
    SetImage,
    SetImageBytes(Bytes),
}

#[widget]
impl Widget for Thumbnail {
    fn model(relm: &Relm<Self>, thumbnail: youtube_feed::Thumbnail) -> ThumbnailModel {
        ThumbnailModel {
            url: thumbnail.url,
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: ThumbnailMsg) {
        match event {
            ThumbnailMsg::SetImage => self.set_image(),
            ThumbnailMsg::SetImageBytes(bytes) => {
                self.set_image_bytes(bytes);
            }
        }
    }

    fn set_image(&mut self) {
        let url = self.model.url.clone();

        let image_id = url.split('/').nth(4);
        let mut cached = false;
        let mut cache_file = None;

        if let Some(id) = image_id {
            let mut user_data_dir =
                glib::get_user_cache_dir().expect("could not get user cache directory");
            user_data_dir.push("tubefeeder");
            user_data_dir.push(&format!("{}.thumbnail", id));
            cache_file = Some(user_data_dir);

            if cache_file.clone().unwrap().exists() {
                cached = true;
            }
        }

        let stream = self.model.relm.stream().clone();

        let (_channel, sender) = Channel::new(move |bytes| {
            stream.emit(ThumbnailMsg::SetImageBytes(bytes));
        });

        thread::spawn(move || {
            if !cached {
                let response = reqwest::blocking::get(&url);

                if response.is_err() {
                    return;
                }

                let parsed = response.unwrap().bytes();

                if parsed.is_err() {
                    return;
                }

                let parsed_bytes = parsed.unwrap();

                sender
                    .send(parsed_bytes.clone())
                    .expect("could not send bytes");

                // Save file to cache.
                if let Some(cache_file) = cache_file {
                    if let Ok(mut cache) = File::create(cache_file) {
                        cache.write_all(&parsed_bytes).unwrap_or(());
                    }
                }
            } else if let Ok(mut cache) = File::open(cache_file.unwrap()) {
                    let mut buffer = vec![];
                    cache.read_to_end(&mut buffer).unwrap_or_default();
                    sender.send(buffer.into()).expect("could not send bytes");
            }
        });
    }

    fn set_image_bytes(&mut self, bytes: Bytes) {
        let glib_bytes = glib::Bytes::from(&bytes.to_vec());
        let stream = MemoryInputStream::from_bytes(&glib_bytes);
        let pixbuf =
            Pixbuf::from_stream_at_scale(&stream, WIDTH, HEIGHT, true, NONE_CANCELLABLE).unwrap();

        self.widgets.image.set_from_pixbuf(Some(&pixbuf));
    }

    fn init_view(&mut self) {
        let pixbuf =
            Pixbuf::new(Colorspace::Rgb, true, 8, WIDTH, HEIGHT).expect("Could not create empty");
        pixbuf.fill(0);

        self.widgets.image.set_from_pixbuf(Some(&pixbuf));
    }

    view! {
        gtk::Box {
            #[name="image"]
            gtk::Image {}
        },
    }
}
