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

use std::cmp::min;

use gdk::{
    gio::{SimpleAction, SimpleActionGroup},
    glib,
    glib::clone,
    prelude::{ActionMapExt, ListModelExt, ObjectExt, ToValue},
    subclass::prelude::ObjectSubclassIsExt,
};
use gtk::{
    traits::{AdjustmentExt, WidgetExt},
    Adjustment,
};
use tf_join::AnyVideo;
use tf_playlist::PlaylistManager;

use super::feed_item_object::VideoObject;

const LOAD_COUNT: usize = 10;

gtk::glib::wrapper! {
    pub struct FeedList(ObjectSubclass<imp::FeedList>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl FeedList {
    fn add_actions(&self) {
        let action_more = SimpleAction::new("more", None);

        action_more.connect_activate(clone!(@strong self as s => move |_, _| {
            let imp = s.imp();
            let items = &imp.items.borrow();
            let model = &imp.model.borrow();
            let loaded_count = &imp.loaded_count.get();

            let to_load = min(LOAD_COUNT, items.len() - loaded_count);

            model.splice(model.n_items(), 0, &items[*loaded_count..(loaded_count + to_load)]);
            imp.loaded_count.set(loaded_count + to_load);

            s.set_more_available();
        }));

        let actions = SimpleActionGroup::new();
        self.insert_action_group("feed", Some(&actions));
        actions.add_action(&action_more);
    }

    fn setup_autoload(&self) {
        let adj = self.imp().scrolled_window.vadjustment();
        adj.connect_changed(clone!(@weak self as s => move |adj| {
            s.load_if_screen_not_filled(adj);
        }));
    }

    fn load_if_screen_not_filled(&self, adj: &Adjustment) {
        if self.property("more-available") && adj.upper() <= adj.page_size() {
            // The screen is not yet filled.
            let _ = self.activate_action("feed.more", None);
        }
    }

    pub fn set_items(&self, new_items: Vec<VideoObject>) {
        let imp = self.imp();
        let items = &imp.items;
        let model = &imp.model;
        let loaded_count = &imp.loaded_count;

        let _ = items.replace(new_items);
        model.borrow().remove_all();
        loaded_count.set(0);

        let _ = self.activate_action("feed.more", None);

        self.set_more_available();
        self.notify("is-empty");
    }

    pub fn prepend(&self, new_item: VideoObject) {
        let imp = self.imp();
        let items = &imp.items;
        let model = &imp.model;
        let loaded_count = &imp.loaded_count;

        let _ = items.borrow_mut().insert(0, new_item.clone());
        model.borrow_mut().insert(0, &new_item);
        loaded_count.set(loaded_count.get() + 1);

        self.set_more_available();
        self.notify("is-empty");
    }

    pub fn remove(&self, new_item: VideoObject) {
        // Extra block needed to end the mutable borrow of `items`.
        {
            let imp = self.imp();
            let mut items = imp.items.borrow_mut();
            let model = &imp.model;
            let loaded_count = &imp.loaded_count;

            if let Some(idx) = items.iter().position(|i| i.video() == new_item.video()) {
                if idx < loaded_count.get() {
                    model.borrow().remove(idx as u32);
                    loaded_count.set(loaded_count.get() - 1);
                }

                items.remove(idx);
            }
        }

        self.set_more_available();
        self.notify("is-empty");
    }

    pub fn set_playlist_manager(&self, playlist_manager: PlaylistManager<String, AnyVideo>) {
        self.imp().playlist_manager.replace(Some(playlist_manager));
        self.imp().setup();
    }

    fn set_more_available(&self) {
        let imp = self.imp();
        let items_count = imp.items.borrow().len();
        let loaded_count = imp.loaded_count.get();

        self.set_property("more-available", (items_count != loaded_count).to_value());
    }
}

pub mod imp {
    use std::cell::{Cell, RefCell};

    use gdk::gio::ListStore;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecBoolean;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::PositionType;
    use gtk::SignalListItemFactory;
    use gtk::Widget;

    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use tf_join::AnyVideo;
    use tf_playlist::PlaylistManager;

    use crate::gui::feed::feed_item::FeedItem;
    use crate::gui::feed::feed_item_object::VideoObject;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/feed_list.ui")]
    pub struct FeedList {
        #[template_child]
        pub(super) feed_list: TemplateChild<gtk::ListView>,
        #[template_child]
        pub(super) scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        pub(super) items: RefCell<Vec<VideoObject>>,
        pub(super) model: RefCell<ListStore>,
        pub(super) loaded_count: Cell<usize>,

        pub(super) playlist_manager: RefCell<Option<PlaylistManager<String, AnyVideo>>>,

        pub(super) more_available: Cell<bool>,
    }

    impl FeedList {
        pub(super) fn setup(&self) {
            let model = gtk::gio::ListStore::new(VideoObject::static_type());
            let selection_model = gtk::NoSelection::new(Some(model.clone()));
            self.feed_list.get().set_model(Some(&selection_model));

            self.model.replace(model);

            let factory = SignalListItemFactory::new();
            let playlist_manager = self
                .playlist_manager
                .borrow()
                .clone()
                .expect("PlaylistManager should be set up");
            factory.connect_setup(move |_, list_item| {
                let feed_item = FeedItem::new(playlist_manager.clone());
                list_item.set_child(Some(&feed_item));

                list_item
                    .property_expression("item")
                    .bind(&feed_item, "video", Widget::NONE);
            });
            self.feed_list.set_factory(Some(&factory));
            self.feed_list.set_single_click_activate(true);

            self.feed_list.connect_activate(move |list_view, position| {
                let model = list_view.model().expect("The model has to exist.");
                let video_object = model
                    .item(position)
                    .expect("The item has to exist.")
                    .downcast::<VideoObject>()
                    .expect("The item has to be an `VideoObject`.");

                video_object.play();
            });

            self.obj().setup_autoload();
        }
    }

    #[gtk::template_callbacks]
    impl FeedList {
        #[template_callback]
        fn edge_reached(&self, pos: PositionType) {
            if pos == PositionType::Bottom {
                let _ = gtk::prelude::WidgetExt::activate_action(
                    self.obj().as_ref(),
                    "feed.more",
                    None,
                );
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FeedList {
        const NAME: &'static str = "TFFeedList";
        type Type = super::FeedList;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FeedList {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().add_actions();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecBoolean::builder("more-available").build(),
                    ParamSpecBoolean::builder("is-empty").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "more-available" => {
                    let value: bool = value
                        .get()
                        .expect("Property more-available of incorrect type");
                    self.more_available.replace(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "more-available" => self.more_available.get().to_value(),
                "is-empty" => (self.model.borrow().n_items() == 0).to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for FeedList {}
    impl BoxImpl for FeedList {}
}
