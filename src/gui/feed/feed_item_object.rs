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

use std::cell::RefCell;

use gdk::glib;
use gdk::subclass::prelude::ObjectSubclassIsExt;
use gdk::{
    glib::{clone, MainContext, Object, PRIORITY_DEFAULT},
    prelude::{Continue, ObjectExt},
};
use tf_core::Video;
use tf_join::AnyVideo;

use crate::downloader::download;
use crate::player::play;

macro_rules! str_prop {
    ( $x:expr ) => {
        ParamSpecString::new($x, $x, $x, None, ParamFlags::READWRITE)
    };
}

macro_rules! prop_set {
    ( $x:expr, $value:expr ) => {
        let input = $value
            .get::<'_, Option<String>>()
            .expect("The value needs to be of type `Option<String>`.");
        $x.replace(input);
    };
}

macro_rules! prop_set_all {
    ( $value:expr, $pspec:expr, $( $key:expr, $element:expr ),* ) => {
        match $pspec.name() {
            $(
                $key => { prop_set!($element, $value); },
            )*
                _ => unimplemented!()
        }
    }
}

macro_rules! prop_get_all {
    ( $pspec:expr, $( $key:expr, $element:expr ),* ) => {
        match $pspec.name() {
            $(
                $key => { $element.borrow().to_value() },
            )*
                _ => unimplemented!()
        }
    }
}

gtk::glib::wrapper! {
    pub struct VideoObject(ObjectSubclass<imp::VideoObject>);
}

impl VideoObject {
    pub fn new(video: AnyVideo) -> Self {
        let s: Self = Object::new(&[
            ("title", &video.title()),
            ("url", &video.url()),
            ("thumbnail-url", &video.thumbnail_url()),
            ("author", &video.subscription().to_string()),
            ("platform", &video.platform().to_string()),
            (
                "date",
                &video
                    .uploaded()
                    .format(&gettextrs::gettext("%F %T"))
                    .to_string(),
            ),
            ("playing", &false),
        ])
        .expect("Failed to create `VideoObject`.");
        s.imp().video.swap(&RefCell::new(Some(video)));
        s
    }

    pub fn video(&self) -> Option<AnyVideo> {
        self.imp().video.borrow().clone()
    }

    pub fn play(&self) {
        self.set_property("playing", true);
        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
        play(
            self.property::<Option<String>>("url").unwrap_or_default(),
            move || {
                let _ = sender.send(());
            },
        );
        receiver.attach(
            None,
            clone!(@weak self as s => @default-return Continue(false), move |_| {
                s.set_property("playing", false);
                Continue(true)
            }),
        );
    }

    pub fn download(&self) {
        self.set_property("downloading", true);
        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
        download(
            self.property::<Option<String>>("url").unwrap_or_default(),
            move || {
                let _ = sender.send(());
            },
        );
        receiver.attach(
            None,
            clone!(@weak self as s => @default-return Continue(false), move |_| {
                s.set_property("downloading", false);
                Continue(true)
            }),
        );
    }
}

mod imp {
    use gtk::glib;
    use std::cell::{Cell, RefCell};
    use tf_join::AnyVideo;

    use gdk::{
        glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString, Value},
        prelude::ToValue,
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    #[derive(Default, Clone)]
    pub struct VideoObject {
        title: RefCell<Option<String>>,
        author: RefCell<Option<String>>,
        platform: RefCell<Option<String>>,
        date: RefCell<Option<String>>,
        url: RefCell<Option<String>>,
        thumbnail_url: RefCell<Option<String>>,

        playing: Cell<bool>,
        downloading: Cell<bool>,

        pub(super) video: RefCell<Option<AnyVideo>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VideoObject {
        const NAME: &'static str = "TFVideoObject";
        type Type = super::VideoObject;
    }

    impl ObjectImpl for VideoObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    str_prop!("title"),
                    str_prop!("url"),
                    str_prop!("thumbnail-url"),
                    str_prop!("author"),
                    str_prop!("platform"),
                    str_prop!("date"),
                    ParamSpecBoolean::new(
                        "playing",
                        "playing",
                        "playing",
                        false,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new(
                        "downloading",
                        "downloading",
                        "downloading",
                        false,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            if pspec.name() == "playing" {
                self.playing
                    .set(value.get().expect("Expect 'playing' to be a boolean."));
                return;
            }
            if pspec.name() == "downloading" {
                self.downloading
                    .set(value.get().expect("Expect 'downloading' to be a boolean."));
                return;
            }
            prop_set_all!(
                value,
                pspec,
                "title",
                self.title,
                "url",
                self.url,
                "thumbnail-url",
                self.thumbnail_url,
                "author",
                self.author,
                "platform",
                self.platform,
                "date",
                self.date
            );
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            if pspec.name() == "playing" {
                return self.playing.get().to_value();
            }
            if pspec.name() == "downloading" {
                return self.downloading.get().to_value();
            }
            prop_get_all!(
                pspec,
                "title",
                self.title,
                "url",
                self.url,
                "thumbnail-url",
                self.thumbnail_url,
                "author",
                self.author,
                "platform",
                self.platform,
                "date",
                self.date
            )
        }
    }
}
