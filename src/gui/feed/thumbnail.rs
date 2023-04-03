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
    glib::{clone, MainContext, PRIORITY_DEFAULT},
    prelude::{Continue, ObjectExt},
    subclass::prelude::ObjectSubclassIsExt,
};

async fn download(thumbnail_url: String) -> Option<image::DynamicImage> {
    log::debug!("Getting thumbnail from url {}", thumbnail_url);
    let response = reqwest::get(&thumbnail_url.clone()).await;

    if response.is_err() {
        log::error!("Failed getting thumbnail for url {}, abort", thumbnail_url);
        return None;
    }

    let parsed = response.unwrap().bytes().await;

    if parsed.is_err() {
        log::error!("Failed getting thumbnail for url {}, abort", thumbnail_url);
        return None;
    }

    let parsed_bytes = parsed.unwrap();

    image::load_from_memory(&parsed_bytes).ok()
}

gtk::glib::wrapper! {
    pub struct Thumbnail(ObjectSubclass<imp::Thumbnail>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl Thumbnail {
    pub fn load_thumbnail(&self) {
        let video = self.imp().video.borrow().clone();
        let thumbnail = &self.imp().thumbnail.clone();

        let thumbnail_url = video
            .as_ref()
            .map(|v| v.property::<Option<String>>("thumbnail-url"))
            .flatten();
        let url = video
            .as_ref()
            .map(|v| v.property::<Option<String>>("url"))
            .flatten();
        if let (Some(thumbnail_url), Some(url)) = (thumbnail_url, url) {
            let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
            tokio::spawn(async move {
                let mut user_cache_dir = gtk::glib::user_cache_dir();
                user_cache_dir.push("tubefeeder");
                user_cache_dir.push(&format!("{}.jpeg", url.replace("/", "_")));
                let path = user_cache_dir;

                if !path.exists() {
                    let image = download(thumbnail_url).await;
                    if let Some(image) = image {
                        if let Err(e) = image.save(&path) {
                            log::error!("Failed to save thumbnail to path {:?}: {}", path, e);
                        }
                    }
                }

                let _ = sender.send(path);
            });

            receiver.attach(
                None,
                clone!(@strong thumbnail => @default-return Continue(false), move |path| {
                    thumbnail.set_filename(Some(&path));
                    Continue(true)
                }),
            );
        }
    }
}

pub mod imp {
    use crate::gui::utility::Utility;

    use std::cell::RefCell;

    use gdk::glib::clone;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use glib::ParamFlags;
    use glib::ParamSpec;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    use crate::gui::feed::feed_item_object::VideoObject;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/thumbnail.ui")]
    pub struct Thumbnail {
        #[template_child]
        pub(super) thumbnail: TemplateChild<gtk::Picture>,
        pub(super) video: RefCell<Option<VideoObject>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Thumbnail {
        const NAME: &'static str = "TFThumbnail";
        type Type = super::Thumbnail;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Thumbnail {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.connect_notify_local(
                Some("video"),
                clone!(@strong obj => move |_, _| {
                    obj.load_thumbnail();
                }),
            );
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "video",
                    "video",
                    "video",
                    Option::<VideoObject>::static_type(),
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "video" => {
                    let value: Option<VideoObject> =
                        value.get().expect("Property video of incorrect type");
                    self.video.replace(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "video" => self.video.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Thumbnail {}
    impl BoxImpl for Thumbnail {}
}
