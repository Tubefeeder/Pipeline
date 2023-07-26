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

use std::sync::{Arc, Mutex};

use gdk::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib::Object;
use tf_filter::FilterGroup;
use tf_join::AnyVideoFilter;

gtk::glib::wrapper! {
    pub struct FilterItem(ObjectSubclass<imp::FilterItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl FilterItem {
    pub fn new(filter_group: Arc<Mutex<FilterGroup<AnyVideoFilter>>>) -> Self {
        let s: Self = Object::builder::<Self>().build();
        s.imp().filter_group.replace(Some(filter_group));
        s
    }
}

pub mod imp {
    use std::cell::RefCell;
    use std::sync::Arc;
    use std::sync::Mutex;

    use gdk::glib::clone;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use glib::ParamSpec;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use tf_filter::FilterGroup;
    use tf_join::AnyVideoFilter;

    use crate::gui::filter::filter_item_object::FilterObject;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/filter_item.ui")]
    pub struct FilterItem {
        #[template_child]
        label_title: TemplateChild<gtk::Label>,
        #[template_child]
        label_channel: TemplateChild<gtk::Label>,
        #[template_child]
        remove: TemplateChild<gtk::Button>,

        filter: RefCell<Option<FilterObject>>,
        pub(super) filter_group: RefCell<Option<Arc<Mutex<FilterGroup<AnyVideoFilter>>>>>,
    }

    impl FilterItem {
        fn bind_remove(&self) {
            let filter = &self.filter;
            let filter_group = &self.filter_group.clone();
            self.remove
                .connect_clicked(clone!(@strong filter, @strong filter_group => move |_| {
                    let filter = filter.borrow().as_ref().map(|s| s.filter()).flatten();
                    if let Some(filter) = filter {
                        let filter_group = filter_group.borrow();
                        filter_group.as_ref().expect("FilterGroup to be set up").lock().expect("FilterGroup to be lockable").remove(&filter);
                    }
                }));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FilterItem {
        const NAME: &'static str = "TFFilterItem";
        type Type = super::FilterItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FilterItem {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecObject::builder::<FilterObject>("filter").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "filter" => {
                    let value: Option<FilterObject> =
                        value.get().expect("Property filter of incorrect type");
                    self.filter.replace(value);
                    self.bind_remove();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "filter" => self.filter.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for FilterItem {}
    impl BoxImpl for FilterItem {}
}
