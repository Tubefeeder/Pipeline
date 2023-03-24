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
use gdk_pixbuf::prelude::SettingsExt;
use gtk::{
    glib::Object,
    traits::{GtkWindowExt, WidgetExt},
};

fn setup_joiner() -> tf_join::Joiner {
    let joiner = tf_join::Joiner::new();
    joiner
}

gtk::glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends libadwaita::ApplicationWindow, gtk::ApplicationWindow, libadwaita::Window, gtk::Window, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &gtk::Application) -> Self {
        Object::new(&[("application", app)]).expect("Failed to create Window")
    }

    pub fn reload(&self) {
        let _ = self.activate_action("win.reload", None);
    }

    fn save_window_size(&self) -> Result<(), gtk::glib::BoolError> {
        let imp = self.imp();

        let (width, height) = self.default_size();

        imp.settings.set_int("window-width", width)?;
        imp.settings.set_int("window-height", height)?;

        imp.settings
            .set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let imp = self.imp();

        let width = imp.settings.int("window-width");
        let height = imp.settings.int("window-height");
        let is_maximized = imp.settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }
}

pub mod imp {
    use crate::config::{APP_ID, PROFILE};

    use std::cell::RefCell;
    use std::sync::Arc;
    use std::sync::Mutex;

    use gdk_pixbuf::glib::clone;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::Inhibit;

    use gtk::CompositeTemplate;
    use libadwaita::subclass::prelude::AdwApplicationWindowImpl;
    use libadwaita::subclass::prelude::AdwWindowImpl;

    use tf_filter::FilterEvent;
    use tf_join::AnySubscriptionList;
    use tf_join::AnyVideo;
    use tf_join::AnyVideoFilter;
    use tf_join::Joiner;
    use tf_join::SubscriptionEvent;
    use tf_observer::Observable;
    use tf_observer::Observer;
    use tf_playlist::PlaylistEvent;
    use tf_playlist::PlaylistManager;

    use crate::csv_file_manager::CsvFileManager;
    use crate::gui::feed::feed_page::FeedPage;
    use crate::gui::filter::filter_page::FilterPage;
    use crate::gui::subscription::subscription_page::SubscriptionPage;
    use crate::gui::watch_later::WatchLaterPage;

    use super::setup_joiner;

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/window.ui")]
    pub struct Window {
        #[template_child]
        pub(in crate::gui) application_stack: TemplateChild<libadwaita::ViewStack>,

        #[template_child]
        pub(in crate::gui) application_stack_bar: TemplateChild<libadwaita::ViewSwitcherBar>,

        pub settings: gtk::gio::Settings,

        #[template_child]
        pub(super) feed_page: TemplateChild<FeedPage>,
        #[template_child]
        pub(super) watchlater_page: TemplateChild<WatchLaterPage>,
        #[template_child]
        pub(super) filter_page: TemplateChild<FilterPage>,
        #[template_child]
        pub(super) subscription_page: TemplateChild<SubscriptionPage>,

        pub(in crate::gui) joiner: RefCell<Option<Joiner>>,
        playlist_manager: RefCell<Option<PlaylistManager<String, AnyVideo>>>,
        any_subscription_list: RefCell<Option<AnySubscriptionList>>,
        _watchlater_file_manager:
            RefCell<Option<Arc<Mutex<Box<dyn Observer<PlaylistEvent<AnyVideo>> + Send>>>>>,
        _subscription_file_manager:
            RefCell<Option<Arc<Mutex<Box<dyn Observer<SubscriptionEvent> + Send>>>>>,
        _filter_file_manager:
            RefCell<Option<Arc<Mutex<Box<dyn Observer<FilterEvent<AnyVideoFilter>> + Send>>>>>,
    }

    impl Default for Window {
        fn default() -> Self {
            Self {
                settings: gtk::gio::Settings::new(APP_ID),
                application_stack: Default::default(),
                application_stack_bar: Default::default(),
                feed_page: Default::default(),
                watchlater_page: Default::default(),
                filter_page: Default::default(),
                subscription_page: Default::default(),
                joiner: Default::default(),
                playlist_manager: Default::default(),
                any_subscription_list: Default::default(),
                _watchlater_file_manager: Default::default(),
                _subscription_file_manager: Default::default(),
                _filter_file_manager: Default::default(),
            }
        }
    }

    impl Window {
        fn setup_feed(&self) {
            self.feed_page.connect_local(
                "add-subscription",
                true,
                clone!(@strong self.subscription_page as s => move |_| {
                    s.present_subscribe();
                    None
                }),
            );
        }
        fn setup_watch_later(&self) {
            let joiner = setup_joiner();
            self.joiner.replace(Some(joiner.clone()));

            let mut watchlater_file_path = glib::user_data_dir();
            watchlater_file_path.push("tubefeeder");
            watchlater_file_path.push("playlist_watch_later.csv");

            let mut playlist_manager = PlaylistManager::new();
            let mut playlist_manager_clone = playlist_manager.clone();

            let _watchlater_file_manager = Arc::new(Mutex::new(Box::new(CsvFileManager::new(
                &watchlater_file_path,
                &mut move |v| {
                    let join_video = joiner.upgrade_video(&v);
                    playlist_manager_clone.toggle(&"WATCHLATER".to_string(), &join_video);
                },
            ))
                as Box<dyn Observer<PlaylistEvent<AnyVideo>> + Send>));

            playlist_manager.attach_at(
                Arc::downgrade(&_watchlater_file_manager),
                &"WATCHLATER".to_string(),
            );

            self.playlist_manager
                .replace(Some(playlist_manager.clone()));
            self._watchlater_file_manager
                .replace(Some(_watchlater_file_manager));
            self.watchlater_page
                .get()
                .set_playlist_manager(playlist_manager);
        }

        fn setup_subscriptions(&self) {
            let joiner = self
                .joiner
                .borrow()
                .clone()
                .expect("Joiner should be set up");

            let mut subscription_list = joiner.subscription_list();

            let mut user_data_dir = gtk::glib::user_data_dir();
            user_data_dir.push("tubefeeder");

            let mut subscriptions_file_path = user_data_dir.clone();
            subscriptions_file_path.push("subscriptions.csv");

            let _subscription_file_manager = Arc::new(Mutex::new(Box::new(CsvFileManager::new(
                &subscriptions_file_path,
                &mut |sub| subscription_list.add(sub),
            ))
                as Box<dyn Observer<SubscriptionEvent> + Send>));

            subscription_list.attach(Arc::downgrade(&_subscription_file_manager));

            self.any_subscription_list
                .replace(Some(subscription_list.clone()));
            self._subscription_file_manager
                .replace(Some(_subscription_file_manager));
            self.subscription_page.get().set_subscription_list(
                subscription_list.clone(),
                self.playlist_manager
                    .borrow()
                    .clone()
                    .expect("PlaylistManager should be set up"),
            );
            self.feed_page.get().setup(
                self.playlist_manager
                    .borrow()
                    .clone()
                    .expect("PlaylistManager should be set up"),
                joiner,
            );

            self.subscription_page.connect_local(
                "subscription-added",
                true,
                clone!(@strong self.feed_page as f => move |_| {
                    f.reload();
                    None
                }),
            );
        }

        fn setup_filter(&self) {
            let joiner = self
                .joiner
                .borrow()
                .clone()
                .expect("Joiner should be set up");
            let filters = joiner.filters();

            let mut user_data_dir = gtk::glib::user_data_dir();
            user_data_dir.push("tubefeeder");

            let mut filters_file_path = user_data_dir.clone();
            filters_file_path.push("filters.csv");

            let _filter_file_manager = Arc::new(Mutex::new(Box::new(CsvFileManager::new(
                &filters_file_path,
                &mut |filter| {
                    filters
                        .lock()
                        .expect("Filter Group to be lockable")
                        .add(filter)
                },
            ))
                as Box<dyn Observer<FilterEvent<AnyVideoFilter>> + Send>));

            filters
                .lock()
                .expect("Filter Group to be lockable")
                .attach(Arc::downgrade(&_filter_file_manager));

            self._filter_file_manager
                .replace(Some(_filter_file_manager));
            self.filter_page.get().set_filter_group(filters);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "TFWindow";
        type Type = super::Window;
        type ParentType = libadwaita::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            // Make sure HeaderBar is loaded.
            crate::gui::header_bar::HeaderBar::ensure_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.setup_feed();
            self.setup_watch_later();
            self.setup_subscriptions();
            self.setup_filter();

            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }
            obj.load_window_size();
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {
        fn close_request(&self, obj: &Self::Type) -> Inhibit {
            let mut user_cache_dir = glib::user_cache_dir();
            user_cache_dir.push("tubefeeder");

            if user_cache_dir.exists() {
                std::fs::remove_dir_all(user_cache_dir).unwrap_or(());
            }

            if let Err(err) = obj.save_window_size() {
                log::warn!("Failed to save window state, {}", &err);
            }

            self.parent_close_request(obj)
        }
    }
    impl ApplicationWindowImpl for Window {}
    impl AdwWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}
