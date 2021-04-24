use crate::youtube_feed;

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

        let stream = self.model.relm.stream().clone();

        let (_channel, sender) = Channel::new(move |bytes| {
            stream.emit(ThumbnailMsg::SetImageBytes(bytes));
        });

        thread::spawn(move || {
            let response = reqwest::blocking::get(&url);

            if response.is_err() {
                return;
            }

            let parsed = response.unwrap().bytes();

            if parsed.is_err() {
                return;
            }

            sender.send(parsed.unwrap()).expect("could not send bytes");
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
