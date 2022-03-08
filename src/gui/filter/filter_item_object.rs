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

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;
use tf_join::AnyVideoFilter;

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
    pub struct FilterObject(ObjectSubclass<imp::FilterObject>);
}

impl FilterObject {
    pub fn new(filter: AnyVideoFilter) -> Self {
        let s: Self = Object::new(&[
            ("title", &filter.title_str().unwrap_or_default().to_string()),
            (
                "channel",
                &filter.subscription_str().unwrap_or_default().to_string(),
            ),
        ])
        .expect("Failed to create `FilterObject`.");
        s.imp().filter.swap(&RefCell::new(Some(filter)));
        s
    }

    pub fn filter(&self) -> Option<AnyVideoFilter> {
        self.imp().filter.borrow().clone()
    }
}

mod imp {
    use gtk::glib;
    use std::cell::RefCell;
    use tf_join::AnyVideoFilter;

    use gdk::{
        glib::{ParamFlags, ParamSpec, ParamSpecString, Value},
        prelude::ToValue,
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    #[derive(Default, Clone)]
    pub struct FilterObject {
        title: RefCell<Option<String>>,
        channel: RefCell<Option<String>>,

        pub(super) filter: RefCell<Option<AnyVideoFilter>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FilterObject {
        const NAME: &'static str = "TFFilterObject";
        type Type = super::FilterObject;
    }

    impl ObjectImpl for FilterObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![str_prop!("title"), str_prop!("channel")]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            prop_set_all!(value, pspec, "title", self.title, "channel", self.channel);
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            prop_get_all!(pspec, "title", self.title, "channel", self.channel)
        }
    }
}
