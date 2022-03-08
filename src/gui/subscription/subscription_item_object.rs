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
        s.imp().setup_name(&s);
        s
    }

    pub fn subscription(&self) -> Option<AnySubscription> {
        self.imp().subscription.borrow().clone()
    }
}

mod imp {
    use gtk::glib;
    use std::cell::RefCell;
    use tf_join::AnySubscription;

    use gdk::{
        glib::{
            clone, MainContext, ParamFlags, ParamSpec, ParamSpecString, Value, PRIORITY_DEFAULT,
        },
        prelude::{Continue, ObjectExt, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    #[derive(Default, Clone)]
    pub struct SubscriptionObject {
        name: RefCell<Option<String>>,
        platform: RefCell<Option<String>>,

        pub(super) subscription: RefCell<Option<AnySubscription>>,
    }

    impl SubscriptionObject {
        pub(super) fn setup_name(&self, obj: &super::SubscriptionObject) {
            let sub_clone = self
                .subscription
                .borrow()
                .clone()
                .expect("Subscription for the item should be set up");

            let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);

            tokio::spawn(async move {
                let client = reqwest::Client::new();
                let name = match &sub_clone {
                    AnySubscription::Youtube(sub) => sub.update_name(&client).await,
                    AnySubscription::Peertube(sub) => sub.update_name(&client).await,
                    AnySubscription::Lbry(sub) => sub.update_name(&client).await,
                };
                if name.is_some() {
                    let _ = sender.send(name);
                }
            });

            receiver.attach(
                None,
                clone!(@strong obj => move |name| {
                    obj.set_property("name", name.to_value());
                    Continue(true)
                }),
            );
        }
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
