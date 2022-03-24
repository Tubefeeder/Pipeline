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

use gdk::subclass::prelude::ObjectSubclassIsExt;
use tf_core::ErrorStore;

gtk::glib::wrapper! {
    pub struct ErrorLabel(ObjectSubclass<imp::ErrorLabel>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl ErrorLabel {
    pub fn set_error_store(&self, error_store: ErrorStore) {
        self.imp().error_store.replace(Some(error_store));
        self.imp().setup(&self);
    }
}

pub mod imp {
    use std::cell::RefCell;
    use std::sync::Arc;
    use std::sync::Mutex;

    use gdk::glib::clone;
    use gdk::glib::MainContext;
    use gdk::glib::ParamFlags;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecString;
    use gdk::glib::Sender;
    use gdk::glib::Value;
    use gdk::glib::PRIORITY_DEFAULT;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use tf_core::ErrorEvent;
    use tf_core::ErrorStore;
    use tf_observer::Observable;
    use tf_observer::Observer;

    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/error_label.ui")]
    pub struct ErrorLabel {
        pub(super) error_store: RefCell<Option<ErrorStore>>,

        error: RefCell<Option<String>>,

        _error_store_observer: RefCell<Option<Arc<Mutex<Box<dyn Observer<ErrorEvent> + Send>>>>>,
    }

    impl ErrorLabel {
        pub(super) fn setup(&self, obj: &super::ErrorLabel) {
            let mut error_store = self
                .error_store
                .borrow()
                .clone()
                .expect("Error Store has to exist");

            let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);

            let observer = Arc::new(Mutex::new(Box::new(ErrorStoreObserver {
                sender: sender.clone(),
            })
                as Box<dyn Observer<ErrorEvent> + Send>));

            error_store.attach(Arc::downgrade(&observer));
            self._error_store_observer.replace(Some(observer));

            receiver.attach(
                None,
                clone!(@strong obj => move |error_event| {
                    match error_event {
                        ErrorEvent::Add(_e) => {
                            let summary = error_store.summary();

                            let message = if summary.network() > 0 {
                                gettextrs::gettext("Error connecting to the network").to_string()
                            } else if summary.parse() > 0 {
                                let msg = gettextrs::ngettext("Error parsing one subscription", "Error parsing {} subscriptions", summary.parse() as u32);
                                msg.replace("{}", &summary.parse().to_string()).to_string()
                            } else {
                                gettextrs::gettext("Some error occured").to_string()
                            };

                            obj.set_property("error", &message);

                        }
                        ErrorEvent::Clear => {
                            obj.set_property("error", "");
                        }
                    }
                    Continue(true)
                }),
            );
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ErrorLabel {
        const NAME: &'static str = "TFErrorLabel";
        type Type = super::ErrorLabel;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ErrorLabel {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecString::new(
                    "error",
                    "error",
                    "error",
                    None,
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "error" => {
                    let value: Option<String> =
                        value.get().expect("Property error of incorrect type");
                    self.error.replace(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "error" => self.error.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ErrorLabel {}
    impl BoxImpl for ErrorLabel {}

    pub struct ErrorStoreObserver {
        sender: Sender<ErrorEvent>,
    }

    impl Observer<ErrorEvent> for ErrorStoreObserver {
        fn notify(&mut self, message: ErrorEvent) {
            let _ = self.sender.send(message);
        }
    }
}
