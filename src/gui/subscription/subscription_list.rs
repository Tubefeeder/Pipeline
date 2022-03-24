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

use gdk::{
    prelude::{Cast, ListModelExtManual},
    subclass::prelude::ObjectSubclassIsExt,
};
use tf_join::AnySubscriptionList;

use super::subscription_item_object::SubscriptionObject;

gtk::glib::wrapper! {
    pub struct SubscriptionList(ObjectSubclass<imp::SubscriptionList>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl SubscriptionList {
    pub fn set(&self, items: Vec<SubscriptionObject>) {
        let imp = self.imp();
        let model = &imp.model.borrow();

        model.remove_all();
        model.splice(0, 0, &items);
    }

    pub fn add(&self, new_item: SubscriptionObject) {
        let imp = self.imp();
        let model = &imp.model;

        model.borrow_mut().insert(0, &new_item);
    }

    pub fn remove(&self, new_item: SubscriptionObject) {
        let imp = self.imp();
        let model = imp.model.borrow();

        if let Some(idx) = model.snapshot().into_iter().position(|i| {
            i.downcast::<SubscriptionObject>()
                .expect("Items should be of type SubscriptionObject")
                .subscription()
                == new_item.subscription()
        }) {
            model.remove(idx as u32);
        }
    }

    pub fn set_subscription_list(&self, subscription_list: AnySubscriptionList) {
        self.imp()
            .any_subscription_list
            .replace(Some(subscription_list));
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
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::SignalListItemFactory;
    use gtk::Widget;

    use gtk::CompositeTemplate;
    use tf_join::AnySubscriptionList;
    use tf_join::SubscriptionEvent;
    use tf_observer::Observable;
    use tf_observer::Observer;

    use crate::gui::subscription::subscription_item::SubscriptionItem;
    use crate::gui::subscription::subscription_item_object::SubscriptionObject;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/subscription_list.ui")]
    pub struct SubscriptionList {
        #[template_child]
        pub(super) subscription_list: TemplateChild<gtk::ListView>,

        pub(super) model: RefCell<ListStore>,

        pub(super) any_subscription_list: RefCell<Option<AnySubscriptionList>>,
        _subscription_observer:
            RefCell<Option<Arc<Mutex<Box<dyn Observer<SubscriptionEvent> + Send>>>>>,
    }

    impl SubscriptionList {
        pub(super) fn setup(&self, obj: &super::SubscriptionList) {
            self.setup_list();
            let mut any_subscription_list = self
                .any_subscription_list
                .borrow()
                .clone()
                .expect("AnySubscriptionList should be set up");

            let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);

            let observer = Arc::new(Mutex::new(Box::new(SubscriptionPageObserver {
                sender: sender.clone(),
            })
                as Box<dyn Observer<SubscriptionEvent> + Send>));

            let existing: Vec<SubscriptionObject> = any_subscription_list
                .iter()
                .map(|v| SubscriptionObject::new(v.clone()))
                .collect();

            any_subscription_list.attach(Arc::downgrade(&observer));
            self._subscription_observer.replace(Some(observer));
            obj.set(existing);

            receiver.attach(
                None,
                clone!(@strong obj => move |subscription_event| {
                    match subscription_event {
                        SubscriptionEvent::Add(s) => {
                            let subscription = SubscriptionObject::new(s);
                            obj.add(subscription);
                        }
                        SubscriptionEvent::Remove(s) => {
                            let subscription = SubscriptionObject::new(s);
                            obj.remove(subscription);
                        }
                    }
                    Continue(true)
                }),
            );
        }

        pub fn setup_list(&self) {
            let model = gtk::gio::ListStore::new(SubscriptionObject::static_type());
            let selection_model = gtk::NoSelection::new(Some(&model));
            self.subscription_list
                .get()
                .set_model(Some(&selection_model));

            self.model.replace(model);

            let factory = SignalListItemFactory::new();
            let any_subscription_list = self
                .any_subscription_list
                .borrow()
                .clone()
                .expect("AnySubscriptionList should be set up");
            factory.connect_setup(move |_, list_item| {
                let subscription_item = SubscriptionItem::new(any_subscription_list.clone());
                list_item.set_child(Some(&subscription_item));

                list_item.property_expression("item").bind(
                    &subscription_item,
                    "subscription",
                    Widget::NONE,
                );
            });
            self.subscription_list.set_factory(Some(&factory));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubscriptionList {
        const NAME: &'static str = "TFSubscriptionList";
        type Type = super::SubscriptionList;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SubscriptionList {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for SubscriptionList {}
    impl BoxImpl for SubscriptionList {}

    pub struct SubscriptionPageObserver {
        sender: Sender<SubscriptionEvent>,
    }

    impl Observer<SubscriptionEvent> for SubscriptionPageObserver {
        fn notify(&mut self, message: SubscriptionEvent) {
            let _ = self.sender.send(message);
        }
    }
}
