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

use gdk::glib::Object;

gtk::glib::wrapper! {
    pub struct HeaderBar(ObjectSubclass<imp::HeaderBar>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl HeaderBar {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create HeaderBar")
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::gio::SimpleAction;
    use gdk::gio::SimpleActionGroup;
    use gdk::glib::clone;
    use gdk::glib::Object;
    use gdk::glib::ParamFlags;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::ParamSpecString;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::Widget;
    use gtk::Builder;
    use libadwaita::AboutWindow;

    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    use crate::gui::import_window::ImportWindow;
    use crate::gui::preferences_window::PreferencesWindow;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/header_bar.ui")]
    pub struct HeaderBar {
        #[template_child]
        child_box: TemplateChild<gtk::Box>,
        #[template_child]
        titlebar: TemplateChild<libadwaita::ViewSwitcherTitle>,

        title: RefCell<Option<String>>,
        child: RefCell<Option<Object>>,
    }

    impl HeaderBar {
        fn setup_actions(&self, obj: &super::HeaderBar) {
            let action_settings = SimpleAction::new("settings", None);
            action_settings.connect_activate(|_, _| {
                let settings = PreferencesWindow::new();
                settings.show();
            });
            let action_import = SimpleAction::new("import", None);
            action_import.connect_activate(clone!(@weak obj => move |_, _| {
                let root = obj
                    .root()
                    .expect("HeaderBar to have root")
                    .downcast::<crate::gui::window::Window>()
                    .expect("Root to be window");
                let import = ImportWindow::new(&root);
                import.show();
            }));

            let action_about = SimpleAction::new("about", None);
            action_about.connect_activate(|_, _| {
                let builder = Builder::from_resource("/ui/about.ui");
                let about: AboutWindow = builder
                    .object("about")
                    .expect("about.ui to have at least one object about");
                about.add_link(&gettextrs::gettext("Donate"), "https://www.tubefeeder.de/donate.html");
                about.show();
            });

            let actions = SimpleActionGroup::new();
            obj.insert_action_group("win", Some(&actions));
            actions.add_action(&action_import);
            actions.add_action(&action_settings);
            actions.add_action(&action_about);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HeaderBar {
        const NAME: &'static str = "TFHeaderBar";
        type Type = super::HeaderBar;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HeaderBar {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.setup_actions(obj);

            obj.connect_root_notify(clone!(@strong self.titlebar as titlebar => move |o| {
                if let Some(root) = o.root() {
                    let window = root
                        .downcast::<crate::gui::window::Window>()
                        .expect("Root to be window");
                    window.connect_realize(clone!(@strong titlebar => move |w| {
                        let stack = &w.imp().application_stack;
                        let stack_switcher = &w.imp().application_stack_bar;
                        titlebar
                            .set_stack(Some(stack));
                        titlebar.bind_property("title-visible", &stack_switcher.get(), "reveal").build();
                    }));
                }
            }));
            obj.connect_notify_local(
                Some("child"),
                clone!(@strong self.child_box as b => move |obj, _| {
                    let widget = obj.property::<Option<Object>>("child");
                    if let Some(widget) = widget {
                        b.append(&widget.dynamic_cast::<Widget>().expect("Child has to be a widget"));
                    }
                }),
            );
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("title", "title", "title", None, ParamFlags::READWRITE),
                    ParamSpecObject::new(
                        "child",
                        "child",
                        "child",
                        Widget::static_type(),
                        ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "title" => {
                    let value: Option<String> =
                        value.get().expect("Property title of incorrect type");
                    self.title.replace(value);
                }
                "child" => {
                    let value: Option<Object> =
                        value.get().expect("Property child of incorrect type");
                    self.child.replace(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "title" => self.title.borrow().to_value(),
                "child" => self.child.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for HeaderBar {}
    impl BoxImpl for HeaderBar {}
}
