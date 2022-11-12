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

use std::sync::{Arc, Mutex};

use gdk::subclass::prelude::ObjectSubclassIsExt;
use tf_filter::FilterGroup;
use tf_join::AnyVideoFilter;

gtk::glib::wrapper! {
    pub struct FilterPage(ObjectSubclass<imp::FilterPage>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl FilterPage {
    pub fn set_filter_group(&self, filter_group: Arc<Mutex<FilterGroup<AnyVideoFilter>>>) {
        self.imp().filter_group.replace(Some(filter_group.clone()));
        self.imp().filter_list.get().set_filter_group(filter_group);
        self.imp().setup_add_filter(&self);
    }
}

pub mod imp {
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::sync::Arc;
    use std::sync::Mutex;

    use gdk::glib::clone;
    use gdk::glib::ParamFlags;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecBoolean;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use regex::Regex;
    use tf_filter::FilterGroup;
    use tf_join::AnyVideoFilter;

    use crate::gui::filter::filter_list::FilterList;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/filter_page.ui")]
    pub struct FilterPage {
        #[template_child]
        pub(super) filter_list: TemplateChild<FilterList>,

        #[template_child]
        pub(super) btn_toggle_add_filter: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) entry_title: TemplateChild<gtk::Entry>,
        #[template_child]
        pub(super) entry_channel: TemplateChild<gtk::Entry>,
        #[template_child]
        pub(super) btn_add_filter: TemplateChild<gtk::Button>,

        pub(super) filter_group: RefCell<Option<Arc<Mutex<FilterGroup<AnyVideoFilter>>>>>,
        add_visible: Cell<bool>,
    }

    impl FilterPage {
        fn setup_toggle_add_filter(&self, obj: &super::FilterPage) {
            self.btn_toggle_add_filter
                .connect_clicked(clone!(@strong obj as s => move |_| {
                    s.set_property("add-visible", !s.property::<bool>("add-visible"));
                }));
        }

        pub(super) fn setup_add_filter(&self, obj: &super::FilterPage) {
            self.btn_add_filter.connect_clicked(clone!(@strong obj as s,
                                                       @strong self.filter_group as filters
                                                       @strong self.entry_title as in_title,
                                                       @strong self.entry_channel as in_channel => move |_| {
                s.set_property("add-visible", !s.property::<bool>("add-visible"));
                let title = in_title.text();
                let channel = in_channel.text();

                in_title.set_text("");
                in_channel.set_text("");

                let title_opt = if title.is_empty() {None} else {Some(title)};
                let channel_opt = if channel.is_empty() {None} else {Some(channel)};

                let title_regex = title_opt.map(|s| Regex::new(&s));
                let channel_regex = channel_opt.map(|s| Regex::new(&s));

                if let Some(Err(_)) = title_regex {
                    // TODO: Error Handling
                    return;
                }
                if let Some(Err(_)) = channel_regex {
                    // TODO: Error Handling
                    return;
                }

                filters
                    .borrow()
                    .as_ref()
                    .expect("Filter List should be set up")
                    .lock()
                    .expect("Filter List should be lockable")
                    .add(AnyVideoFilter::new(None,
                                            title_regex.map(|r| r.unwrap()),
                                            channel_regex.map(|r| r.unwrap())
                                            ).into()
                         );

            }));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FilterPage {
        const NAME: &'static str = "TFFilterPage";
        type Type = super::FilterPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FilterPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.setup_toggle_add_filter(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecBoolean::new(
                    "add-visible",
                    "add-visible",
                    "add-visible",
                    false,
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "add-visible" => {
                    let _ =
                        self.add_visible.replace(value.get().expect(
                            "The property 'add-visible' of TFFilterPage has to be boolean",
                        ));
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "add-visible" => self.add_visible.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for FilterPage {}
    impl BoxImpl for FilterPage {}
}
