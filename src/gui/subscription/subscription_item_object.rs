/*
 * Copyright 2021 - 2022 Julian Schmidhuber <github@schmiddi.anonaddy.com>
 *
 * This file is part of Pipeline.
 *
 * Pipeline is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Pipeline is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Pipeline.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;
use gdk_pixbuf::prelude::ObjectExt;
use tf_core::Subscription;
use tf_join::AnySubscription;

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
    pub struct SubscriptionObject(ObjectSubclass<imp::SubscriptionObject>);
}

impl SubscriptionObject {
    pub fn new(subscription: AnySubscription) -> Self {
        let s: Self = Object::new(&[
            ("name", &subscription.to_string()),
            ("platform", &subscription.platform().to_string()),
        ])
        .expect("Failed to create `SubscriptionObject`.");
        s.imp().subscription.swap(&RefCell::new(Some(subscription)));
        s
    }

    pub fn subscription(&self) -> Option<AnySubscription> {
        self.imp().subscription.borrow().clone()
    }

    pub fn update_name(&self, sub: &AnySubscription) {
        self.set_property("name", sub.name());
    }
}

mod imp {
    use gtk::glib;
    use std::cell::RefCell;
    use tf_join::AnySubscription;

    use gdk::{
        glib::{ParamFlags, ParamSpec, ParamSpecString, Value},
        prelude::ToValue,
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    #[derive(Default, Clone)]
    pub struct SubscriptionObject {
        name: RefCell<Option<String>>,
        platform: RefCell<Option<String>>,

        pub(super) subscription: RefCell<Option<AnySubscription>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubscriptionObject {
        const NAME: &'static str = "TFSubscriptionObject";
        type Type = super::SubscriptionObject;
    }

    impl ObjectImpl for SubscriptionObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![str_prop!("name"), str_prop!("platform")]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            prop_set_all!(value, pspec, "name", self.name, "platform", self.platform);
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            prop_get_all!(pspec, "name", self.name, "platform", self.platform)
        }
    }
}
