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
use gdk_pixbuf::prelude::Cast;
use gtk::traits::WidgetExt;
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
    }

    fn window(&self) -> crate::gui::window::Window {
        self.root()
            .expect("FilterPage to have root")
            .downcast::<crate::gui::window::Window>()
            .expect("Root to be window")
    }
}

pub mod imp {
    use std::cell::RefCell;
    use std::sync::Arc;
    use std::sync::Mutex;

    use gdk::glib::clone;
    use gdk::glib::ParamSpec;
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
        pub(super) dialog_add: TemplateChild<libadwaita::MessageDialog>,

        pub(super) filter_group: RefCell<Option<Arc<Mutex<FilterGroup<AnyVideoFilter>>>>>,
    }

    #[gtk::template_callbacks]
    impl FilterPage {
        fn setup_toggle_add_filter(&self, obj: &super::FilterPage) {
            self.btn_toggle_add_filter.connect_clicked(
                clone!(@strong obj as s, @strong self.dialog_add as dialog, @strong self.entry_title as in_title, @strong self.entry_channel as in_channel => move |_| {
                    in_title.set_text("");
                    in_channel.set_text("");

                    // Theoretically only needs to be done once, but when setting up the page does
                    // not yet have a root.
                    let window = s.window();
                    dialog.set_transient_for(Some(&window));
                    dialog.present();
                }),
            );
        }

        #[template_callback]
        fn handle_add_filter(&self, response: Option<&str>) {
            if response != Some("add") {
                return;
            }

            let in_title = &self.entry_title;
            let in_channel = &self.entry_channel;
            let filters = &self.filter_group;

            let title = in_title.text();
            let channel = in_channel.text();

            in_title.set_text("");
            in_channel.set_text("");

            let title_opt = if title.is_empty() { None } else { Some(title) };
            let channel_opt = if channel.is_empty() {
                None
            } else {
                Some(channel)
            };

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
                .add(
                    AnyVideoFilter::new(
                        None,
                        title_regex.map(|r| r.unwrap()),
                        channel_regex.map(|r| r.unwrap()),
                    )
                    .into(),
                );
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FilterPage {
        const NAME: &'static str = "TFFilterPage";
        type Type = super::FilterPage;
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

    impl ObjectImpl for FilterPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.setup_toggle_add_filter(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(Vec::new);
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            _value: &glib::Value,
            _pspec: &glib::ParamSpec,
        ) {
            unimplemented!()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, _pspec: &glib::ParamSpec) -> glib::Value {
            unimplemented!()
        }
    }

    impl WidgetImpl for FilterPage {}
    impl BoxImpl for FilterPage {}
}
