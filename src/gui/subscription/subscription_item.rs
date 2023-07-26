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

use gdk::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib::Object;
use tf_join::AnySubscriptionList;

gtk::glib::wrapper! {
    pub struct SubscriptionItem(ObjectSubclass<imp::SubscriptionItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl SubscriptionItem {
    pub fn new(subscription_list: AnySubscriptionList) -> Self {
        let s: Self = Object::builder().build();
        s.imp().subscription_list.replace(Some(subscription_list));
        s
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::clone;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use gdk_pixbuf::glib::subclass::Signal;
    use glib::subclass::InitializingObject;
    use glib::ParamSpec;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use tf_join::AnySubscriptionList;

    use crate::gui::subscription::subscription_item_object::SubscriptionObject;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/subscription_item.ui")]
    pub struct SubscriptionItem {
        #[template_child]
        label_name: TemplateChild<gtk::Label>,
        #[template_child]
        label_platform: TemplateChild<gtk::Label>,
        #[template_child]
        remove: TemplateChild<gtk::Button>,

        subscription: RefCell<Option<SubscriptionObject>>,
        pub(super) subscription_list: RefCell<Option<AnySubscriptionList>>,
    }

    #[gtk::template_callbacks]
    impl SubscriptionItem {
        fn bind_remove(&self) {
            let subscription = &self.subscription;
            let subscription_list = &self.subscription_list;
            self.remove.connect_clicked(
                clone!(@strong subscription, @strong subscription_list => move |_| {
                    let subscription = subscription.borrow().as_ref().map(|s| s.subscription()).flatten();
                    if let Some(subscription) = subscription {
                        let mut subscription_list = subscription_list.borrow_mut();
                        subscription_list.as_mut().unwrap().remove(subscription);
                    }
                }),
            );
        }

        #[template_callback]
        fn handle_go_to_videos(&self) {
            if let Some(sub) = self.subscription.borrow().as_ref() {
                self.obj().emit_by_name::<()>("go-to-videos", &[&sub]);
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubscriptionItem {
        const NAME: &'static str = "TFSubscriptionItem";
        type Type = super::SubscriptionItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl SubscriptionItem {}

    impl ObjectImpl for SubscriptionItem {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::builder::<SubscriptionObject>("subscription").build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "subscription" => {
                    let value: Option<SubscriptionObject> = value
                        .get()
                        .expect("Property subscription of incorrect type");
                    self.subscription.replace(value);
                    self.bind_remove();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "subscription" => self.subscription.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("go-to-videos")
                    .param_types([SubscriptionObject::static_type()])
                    .build()]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for SubscriptionItem {}
    impl BoxImpl for SubscriptionItem {}
}
