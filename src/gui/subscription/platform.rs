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
use tf_join::Platform;

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
    pub struct PlatformObject(ObjectSubclass<imp::PlatformObject>);
}

impl PlatformObject {
    pub fn new(platform: Platform) -> Self {
        let s: Self = Object::new(&[("name", &platform.to_string())])
            .expect("Failed to create `PlatformObject`.");
        s.imp().platform.swap(&RefCell::new(Some(platform)));
        s
    }

    pub fn platform(&self) -> Option<Platform> {
        self.imp().platform.borrow().clone()
    }
}

mod imp {
    use gtk::glib;
    use std::cell::RefCell;
    use tf_join::Platform;

    use gdk::{
        glib::{ParamFlags, ParamSpec, ParamSpecString, Value},
        prelude::ToValue,
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    #[derive(Default, Clone)]
    pub struct PlatformObject {
        name: RefCell<Option<String>>,

        pub(super) platform: RefCell<Option<Platform>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlatformObject {
        const NAME: &'static str = "TFPlatformObject";
        type Type = super::PlatformObject;
    }

    impl ObjectImpl for PlatformObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| vec![str_prop!("name")]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            prop_set_all!(value, pspec, "name", self.name);
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            prop_get_all!(pspec, "name", self.name)
        }
    }
}
