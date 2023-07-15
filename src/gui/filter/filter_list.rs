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

use gdk::{
    prelude::{Cast, ListModelExtManual},
    subclass::prelude::ObjectSubclassIsExt,
};
use gdk_pixbuf::prelude::ObjectExt;
use tf_filter::FilterGroup;
use tf_join::AnyVideoFilter;

use super::filter_item_object::FilterObject;

gtk::glib::wrapper! {
    pub struct FilterList(ObjectSubclass<imp::FilterList>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl FilterList {
    pub fn set(&self, items: Vec<FilterObject>) {
        let imp = self.imp();
        let model = &imp.model.borrow();

        model.remove_all();
        model.splice(0, 0, &items);
        self.notify("is-empty");
    }

    pub fn add(&self, new_item: FilterObject) {
        let imp = self.imp();
        let model = &imp.model;

        model.borrow_mut().insert(0, &new_item);
        self.notify("is-empty");
    }

    pub fn remove(&self, new_item: FilterObject) {
        let imp = self.imp();
        let model = imp.model.borrow();

        if let Some(idx) = model.snapshot().into_iter().position(|i| {
            i.downcast::<FilterObject>()
                .expect("Items should be of type FilterObject")
                .filter()
                == new_item.filter()
        }) {
            model.remove(idx as u32);
        }
        self.notify("is-empty");
    }

    pub fn set_filter_group(&self, filter_group: Arc<Mutex<FilterGroup<AnyVideoFilter>>>) {
        self.imp().filter_group.replace(Some(filter_group));
        self.imp().setup(&self);
    }
}

pub mod imp {
    use std::cell::RefCell;
    use std::sync::Arc;
    use std::sync::Mutex;

    use gdk::gio::ListStore;
    use gdk::glib::clone;
    use gdk::glib::MainContext;
    use gdk::glib::Sender;
    use gdk::glib::PRIORITY_DEFAULT;
    use gdk_pixbuf::glib::ParamFlags;
    use gdk_pixbuf::glib::ParamSpec;
    use gdk_pixbuf::glib::ParamSpecBoolean;
    use gdk_pixbuf::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::SignalListItemFactory;
    use gtk::Widget;

    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use tf_filter::FilterEvent;
    use tf_filter::FilterGroup;
    use tf_join::AnyVideoFilter;
    use tf_observer::Observable;
    use tf_observer::Observer;

    use crate::gui::filter::filter_item::FilterItem;
    use crate::gui::filter::filter_item_object::FilterObject;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/filter_list.ui")]
    pub struct FilterList {
        #[template_child]
        pub(super) filter_list: TemplateChild<gtk::ListView>,

        pub(super) model: RefCell<ListStore>,

        pub(super) filter_group: RefCell<Option<Arc<Mutex<FilterGroup<AnyVideoFilter>>>>>,
        _filter_observer:
            RefCell<Option<Arc<Mutex<Box<dyn Observer<FilterEvent<AnyVideoFilter>> + Send>>>>>,
    }

    impl FilterList {
        pub(super) fn setup(&self, obj: &super::FilterList) {
            self.setup_list();
            let filter_group = self
                .filter_group
                .borrow()
                .clone()
                .expect("FilterGroup should be set up");

            let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);

            let observer = Arc::new(Mutex::new(Box::new(FilterPageObserver {
                sender: sender.clone(),
            })
                as Box<dyn Observer<FilterEvent<AnyVideoFilter>> + Send>));

            let mut filter_group = filter_group.lock().unwrap();
            let existing: Vec<FilterObject> = filter_group
                .iter()
                .map(|v| FilterObject::new(v.clone()))
                .collect();

            filter_group.attach(Arc::downgrade(&observer));
            self._filter_observer.replace(Some(observer));
            obj.set(existing);

            receiver.attach(
                None,
                clone!(@strong obj => move |filter_event| {
                    match filter_event {
                        FilterEvent::Add(s) => {
                            let filter = FilterObject::new(s);
                            obj.add(filter);
                        }
                        FilterEvent::Remove(s) => {
                            let filter = FilterObject::new(s);
                            obj.remove(filter);
                        }
                    }
                    Continue(true)
                }),
            );
        }

        pub fn setup_list(&self) {
            let model = gtk::gio::ListStore::new(FilterObject::static_type());
            let selection_model = gtk::NoSelection::new(Some(&model));
            self.filter_list.get().set_model(Some(&selection_model));

            self.model.replace(model);

            let factory = SignalListItemFactory::new();
            let filter_group = self
                .filter_group
                .borrow()
                .clone()
                .expect("FilterGroup should be set up");
            factory.connect_setup(move |_, list_item| {
                let filter_item = FilterItem::new(filter_group.clone());
                list_item.set_child(Some(&filter_item));

                list_item
                    .property_expression("item")
                    .bind(&filter_item, "filter", Widget::NONE);
            });
            self.filter_list.set_factory(Some(&factory));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FilterList {
        const NAME: &'static str = "TFFilterList";
        type Type = super::FilterList;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FilterList {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecBoolean::new(
                    "is-empty",
                    "is-empty",
                    "is-empty",
                    false,
                    ParamFlags::READABLE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, _value: &Value, _pspec: &ParamSpec) {
            unimplemented!()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "is-empty" => (self.model.borrow().n_items() == 0).to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for FilterList {}
    impl BoxImpl for FilterList {}

    pub struct FilterPageObserver {
        sender: Sender<FilterEvent<AnyVideoFilter>>,
    }

    impl Observer<FilterEvent<AnyVideoFilter>> for FilterPageObserver {
        fn notify(&mut self, message: FilterEvent<AnyVideoFilter>) {
            let _ = self.sender.send(message);
        }
    }
}
