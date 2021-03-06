use crate::youtube_feed::feed;

use std::thread;

use bytes::Bytes;
use gdk_pixbuf::Pixbuf;
use gio::{MemoryInputStream, NONE_CANCELLABLE};
use gtk::prelude::*;
use gtk::Image;
use relm::{Channel, Relm, Widget};
use relm_derive::{widget, Msg};

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
    fn model(relm: &Relm<Self>, thumbnail: feed::Thumbnail) -> ThumbnailModel {
        ThumbnailModel {
            url: thumbnail.url,
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: ThumbnailMsg) {
        match event {
            ThumbnailMsg::SetImage => {
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
            ThumbnailMsg::SetImageBytes(bytes) => {
                let image_box = self.widgets.image_box.clone();

                let glib_bytes = glib::Bytes::from(&bytes.to_vec());
                let stream = MemoryInputStream::from_bytes(&glib_bytes);
                let pixbuf =
                    Pixbuf::from_stream_at_scale(&stream, 120, 90, true, NONE_CANCELLABLE).unwrap();
                let image = Image::from_pixbuf(Some(&pixbuf));

                image_box.add(&image);

                image_box.show_all();
            }
        }
    }

    view! {
        #[name="image_box"]
        gtk::Box {
        },
    }
}
