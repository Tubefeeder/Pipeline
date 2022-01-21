/*
 * Copyright 2021 Julian Schmidhuber <github@schmiddi.anonaddy.com>
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

use crate::csv_file_manager::CsvFileManager;
use crate::gui::feed::FeedPageMsg;

use tubefeeder_derive::FromUiResource;

use tf_core::ErrorStore;
use tf_filter::{FilterEvent, FilterGroup};
use tf_join::{AnySubscriptionList, AnyVideo, AnyVideoFilter, Joiner, SubscriptionEvent};
use tf_observer::{Observable, Observer};
use tf_playlist::{PlaylistEvent, PlaylistManager};

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use gtk::prelude::*;
use relm::{AppUpdate, Components, Model, RelmComponent, Widgets};

use super::feed::FeedPageModel;
use super::header_bar::HeaderBarModel;

/// The ration between the fonts of the title and the channel/date.
pub const FONT_RATIO: f32 = 2.0 / 3.0;

pub fn get_font_size() -> i32 {
    gtk::Settings::default()
        .unwrap()
        .gtk_font_name()
        .unwrap_or_else(|| " ".into())
        .to_string()
        .split(' ')
        .last()
        .unwrap_or("")
        .parse::<i32>()
        .unwrap_or(12)
}

fn init_icons<P: IsA<gdk::Display>>(display: &P) {
    let icon_theme = gtk::IconTheme::for_display(display);

    icon_theme.add_resource_path("/");
    icon_theme.add_resource_path("/org/gnome/design/IconLibrary/data/icons/");
}

pub fn migrate_config(old: &PathBuf, new: &PathBuf) {
    if old.exists() {
        let old_file_res = OpenOptions::new().read(true).write(false).open(old);

        if old_file_res.is_err() {
            log::error!("A error migrating configuration occured: Cannot open old file");
            return;
        }

        let mut old_str = String::new();
        if old_file_res.unwrap().read_to_string(&mut old_str).is_err() {
            log::error!("A error migrating configuration occured: Cannot read from old file");
            return;
        }

        let new_str = old_str
            .replace("https://www.youtube.com/channel/", "")
            .replace("+00:00", "")
            .lines()
            .skip(1)
            .map(|s| format!("{},{}\n", String::from(tf_join::Platform::Youtube), s))
            .collect::<String>();

        let new_file_res = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(new);
        if new_file_res.is_err() {
            log::error!("A error migrating configuration occured: Cannot open new file");
            return;
        }
        if write!(&mut new_file_res.unwrap(), "{}", new_str).is_err() {
            log::error!("A error migrating configuration occured: Cannot write to new file");
            return;
        }
    }
}

#[derive(Debug)]
pub enum AppMsg {
    Loading(bool),
    Reload,
    ToggleAddSubscription,
    ToggleAddFilter,
    Quit,
}

pub struct AppModel {
    pub(crate) joiner: Joiner,
    pub(crate) playlist_manager: PlaylistManager<String, AnyVideo>,
    pub(crate) errors: ErrorStore,

    _subscription_file_manager: Arc<Mutex<Box<dyn Observer<SubscriptionEvent> + Send>>>,
    _filter_file_manager: Arc<Mutex<Box<dyn Observer<FilterEvent<AnyVideoFilter>> + Send>>>,
    _watchlater_file_manager: Arc<Mutex<Box<dyn Observer<PlaylistEvent<AnyVideo>> + Send>>>,
    subscription_list: AnySubscriptionList,
    filters: Arc<Mutex<FilterGroup<AnyVideoFilter>>>,
    loading: bool,
}

#[derive(FromUiResource)]
pub struct AppWidgets {
    window: gtk::ApplicationWindow,
    header_bar: gtk::Box,
    error_label: gtk::Box,
    application_stack: libadwaita::ViewStack,
    view_switcher_box: gtk::Box,
    feed_page: gtk::Box,
    watch_later_page: gtk::Box,
    filter_page: gtk::Box,
    subscription_page: gtk::Box,
}

pub struct AppComponents {
    header_bar: RelmComponent<HeaderBarModel, AppModel>,
    feed_page: RelmComponent<FeedPageModel, AppModel>,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

impl AppUpdate for AppModel {
    fn update(
        &mut self,
        msg: Self::Msg,
        components: &Self::Components,
        _sender: relm::Sender<Self::Msg>,
    ) -> bool {
        log::debug!("Got Message {:?}", msg);
        match msg {
            AppMsg::Loading(_) => todo!(),
            AppMsg::Reload => {
                let _ = components.feed_page.send(FeedPageMsg::Reload);
            }
            AppMsg::ToggleAddSubscription => todo!(),
            AppMsg::ToggleAddFilter => todo!(),
            AppMsg::Quit => todo!(),
        }
        true
    }
}

impl Widgets<AppModel, ()> for AppWidgets {
    type Root = gtk::ApplicationWindow;

    fn init_view(
        _model: &AppModel,
        components: &AppComponents,
        _sender: relm::Sender<AppMsg>,
    ) -> Self {
        let widgets = AppWidgets::from_resource("/ui/window.ui");
        init_icons(&widgets.window.display());

        widgets
            .header_bar
            .append(components.header_bar.root_widget());
        widgets.feed_page.append(components.feed_page.root_widget());

        widgets
    }

    fn root_widget(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(&mut self, _model: &AppModel, _sender: relm::Sender<AppMsg>) {
        // TODO
    }
}

impl Components<AppModel> for AppComponents {
    fn init_components(parent_model: &AppModel, parent_sender: relm::Sender<AppMsg>) -> Self {
        AppComponents {
            header_bar: RelmComponent::new(parent_model, parent_sender.clone()),
            feed_page: RelmComponent::new(parent_model, parent_sender),
        }
    }

    fn connect_parent(&mut self, _parent_widgets: &AppWidgets) {}
}

impl AppModel {
    pub fn new(joiner: Joiner) -> Self {
        let mut user_cache_dir = glib::user_cache_dir();
        user_cache_dir.push("tubefeeder");

        if !user_cache_dir.exists() {
            std::fs::create_dir_all(user_cache_dir).expect("could not create user cache dir");
        }

        let mut user_data_dir = glib::user_data_dir();
        user_data_dir.push("tubefeeder");

        if !user_data_dir.exists() {
            std::fs::create_dir_all(user_data_dir.clone()).expect("could not create user data dir");
        }

        let mut subscriptions_file_path = user_data_dir.clone();
        subscriptions_file_path.push("subscriptions.csv");

        if !subscriptions_file_path.exists() {
            let mut old_file_path = user_data_dir.clone();
            old_file_path.push("subscriptions.db");
            migrate_config(&old_file_path, &subscriptions_file_path);
        }

        let mut filter_file_path = user_data_dir.clone();
        filter_file_path.push("filters.csv");

        if !filter_file_path.exists() {
            let mut old_file_path = user_data_dir.clone();
            old_file_path.push(&"filters.db");
            migrate_config(&old_file_path, &filter_file_path);
        }

        let mut watchlater_file_path = user_data_dir.clone();
        watchlater_file_path.push("playlist_watch_later.csv");

        if !watchlater_file_path.exists() {
            let mut old_file_path = user_data_dir.clone();
            old_file_path.push("watch_later.db");
            migrate_config(&old_file_path, &watchlater_file_path);
        }

        let mut subscription_list = joiner.subscription_list();
        let filters = joiner.filters();
        let mut playlist_manager = PlaylistManager::new();
        let mut playlist_manager_clone = playlist_manager.clone();
        let joiner_clone = joiner.clone();

        let _subscription_file_manager = Arc::new(Mutex::new(Box::new(CsvFileManager::new(
            &subscriptions_file_path,
            &mut |sub| subscription_list.add(sub),
        ))
            as Box<dyn Observer<SubscriptionEvent> + Send>));

        let _filter_file_manager = Arc::new(Mutex::new(Box::new(CsvFileManager::new(
            &filter_file_path,
            &mut |fil| filters.lock().unwrap().add(fil),
        ))
            as Box<dyn Observer<FilterEvent<AnyVideoFilter>> + Send>));

        let _watchlater_file_manager = Arc::new(Mutex::new(Box::new(CsvFileManager::new(
            &watchlater_file_path,
            &mut move |v| {
                let join_video = joiner_clone.upgrade_video(&v);
                playlist_manager_clone.toggle(&"WATCHLATER".to_string(), &join_video);
            },
        ))
            as Box<dyn Observer<PlaylistEvent<AnyVideo>> + Send>));

        subscription_list.attach(Arc::downgrade(&_subscription_file_manager));

        filters
            .lock()
            .unwrap()
            .attach(Arc::downgrade(&_filter_file_manager));

        playlist_manager.attach_at(
            Arc::downgrade(&_watchlater_file_manager),
            &"WATCHLATER".to_string(),
        );

        AppModel {
            _subscription_file_manager,
            _filter_file_manager,
            _watchlater_file_manager,
            subscription_list,
            filters,
            loading: false,
            joiner,
            playlist_manager,
            errors: ErrorStore::new(),
        }
    }
}

// #[widget]
// impl Widget for Win {
//     fn init_view(&mut self) {
//         self.widgets.window.resize(800, 500);

//         // Build view switcher
//         let view_switcher = ViewSwitcherBarBuilder::new()
//             .stack(&self.widgets.application_stack)
//             .reveal(true)
//             .build();

//         self.widgets.view_switcher_box.add(&view_switcher);
//         view_switcher.show();

//         // Build header bar
//         let header_bar_stream = self.components.header_bar.stream();
//         header_bar_stream.emit(HeaderBarMsg::SetPage(Page::Feed));

//         self.widgets
//             .application_stack
//             .connect_visible_child_notify(move |stack| {
//                 let child = stack.visible_child().unwrap();
//                 let title = child.widget_name();
//                 header_bar_stream.emit(HeaderBarMsg::SetPage(Page::from_str(&title).unwrap()));
//             });

//         self.widgets.loading_spinner.start();

//         self.model.app_stream.emit(AppMsg::Reload);
//     }

//     fn model(relm: &Relm<Self>, joiner: Joiner) -> AppModel {
//         init_icons();

//         let mut user_cache_dir = glib::user_cache_dir();
//         user_cache_dir.push("tubefeeder");

//         if !user_cache_dir.exists() {
//             std::fs::create_dir_all(user_cache_dir).expect("could not create user cache dir");
//         }

//         let mut user_data_dir = glib::user_data_dir();
//         user_data_dir.push("tubefeeder");

//         if !user_data_dir.exists() {
//             std::fs::create_dir_all(user_data_dir.clone()).expect("could not create user data dir");
//         }

//         let mut subscriptions_file_path = user_data_dir.clone();
//         subscriptions_file_path.push("subscriptions.csv");

//         if !subscriptions_file_path.exists() {
//             let mut old_file_path = user_data_dir.clone();
//             old_file_path.push("subscriptions.db");
//             migrate_config(&old_file_path, &subscriptions_file_path);
//         }

//         let mut filter_file_path = user_data_dir.clone();
//         filter_file_path.push("filters.csv");

//         if !filter_file_path.exists() {
//             let mut old_file_path = user_data_dir.clone();
//             old_file_path.push(&"filters.db");
//             migrate_config(&old_file_path, &filter_file_path);
//         }

//         let mut watchlater_file_path = user_data_dir.clone();
//         watchlater_file_path.push("playlist_watch_later.csv");

//         if !watchlater_file_path.exists() {
//             let mut old_file_path = user_data_dir.clone();
//             old_file_path.push("watch_later.db");
//             migrate_config(&old_file_path, &watchlater_file_path);
//         }

//         let mut subscription_list = joiner.subscription_list();
//         let filters = joiner.filters();
//         let mut playlist_manager = PlaylistManager::new();
//         let mut playlist_manager_clone = playlist_manager.clone();
//         let joiner_clone = joiner.clone();

//         let _subscription_file_manager = Arc::new(Mutex::new(Box::new(CsvFileManager::new(
//             &subscriptions_file_path,
//             &mut |sub| subscription_list.add(sub),
//         ))
//             as Box<dyn Observer<SubscriptionEvent> + Send>));

//         let _filter_file_manager = Arc::new(Mutex::new(Box::new(CsvFileManager::new(
//             &filter_file_path,
//             &mut |fil| filters.lock().unwrap().add(fil),
//         ))
//             as Box<dyn Observer<FilterEvent<AnyVideoFilter>> + Send>));

//         let _watchlater_file_manager = Arc::new(Mutex::new(Box::new(CsvFileManager::new(
//             &watchlater_file_path,
//             &mut move |v| {
//                 let join_video = joiner_clone.upgrade_video(&v);
//                 playlist_manager_clone.toggle(&"WATCHLATER".to_string(), &join_video);
//             },
//         ))
//             as Box<dyn Observer<PlaylistEvent<AnyVideo>> + Send>));

//         subscription_list.attach(Arc::downgrade(&_subscription_file_manager));

//         filters
//             .lock()
//             .unwrap()
//             .attach(Arc::downgrade(&_filter_file_manager));

//         playlist_manager.attach_at(
//             Arc::downgrade(&_watchlater_file_manager),
//             &"WATCHLATER".to_string(),
//         );

//         AppModel {
//             app_stream: relm.stream().clone(),
//             _subscription_file_manager,
//             _filter_file_manager,
//             _watchlater_file_manager,
//             subscription_list,
//             filters,
//             loading: false,
//             joiner,
//             playlist_manager,
//             errors: ErrorStore::new(),
//         }
//     }

//     fn update(&mut self, event: AppMsg) {
//         match event {
//             AppMsg::Loading(loading) => {
//                 self.model.loading = loading;
//             }
//             AppMsg::Reload => {
//                 self.reload();
//             }
//             AppMsg::ToggleAddSubscription => {
//                 self.components
//                     .subscriptions_page
//                     .emit(SubscriptionsPageMsg::ToggleAddSubscription);
//             }
//             AppMsg::ToggleAddFilter => {
//                 self.components
//                     .filter_page
//                     .emit(FilterPageMsg::ToggleAddFilter);
//             }
//             AppMsg::Quit => {
//                 gtk::main_quit();

//                 let mut user_cache_dir = glib::user_cache_dir();
//                 user_cache_dir.push("tubefeeder");

//                 if user_cache_dir.exists() {
//                     std::fs::remove_dir_all(user_cache_dir).unwrap_or(());
//                 }
//             }
//         }
//     }

//     fn reload(&mut self) {
//         let loading_spinner = self.widgets.loading_spinner.clone();
//         loading_spinner.set_visible(true);

//         let feed_stream = self.components.feed_page.stream();
//         let app_stream = self.model.app_stream.clone();
//         app_stream.emit(AppMsg::Loading(true));

//         let (_channel, sender) = relm::Channel::new(move |feed: std::vec::IntoIter<AnyVideo>| {
//             feed_stream.emit(FeedPageMsg::SetFeed(Box::new(feed)));
//             app_stream.emit(AppMsg::Loading(false));
//         });

//         let joiner = self.model.joiner.clone();
//         let errors = self.model.errors.clone();
//         errors.clear();
//         tokio::spawn(async move {
//             let feed = joiner.generate(&errors).await;
//             sender.send(feed).unwrap()
//         });
//     }

//     view! {
//         #[name="window"]
//         gtk::Window {
//             titlebar: view! {
//                 #[name="header_bar"]
//                 HeaderBar(self.model.app_stream.clone()) {
//                 }
//             },
//             #[name="view_switcher_box"]
//             gtk::Box {
//                 gtk::Box {
//                     orientation: Vertical,
//                     #[name="error_label"]
//                     ErrorLabel(self.model.errors.clone()) {},
//                     #[name="loading_spinner"]
//                     gtk::Spinner {
//                         visible: self.model.loading,
//                         active: true
//                     }
//                 },
//                 orientation: Vertical,
//                 #[name="application_stack"]
//                 gtk::Stack {
//                     #[name="feed_page"]
//                     FeedPage(self.model.playlist_manager.clone()) {
//                         widget_name: &String::from(Page::Feed),
//                         child: {
//                             icon_name: Some("go-home-symbolic"),
//                             title: Some(&String::from(Page::Feed))
//                         }
//                     },
//                     #[name="watch_later_page"]
//                     PlaylistPage(self.model.playlist_manager.clone(), "WATCHLATER".to_string()) {
//                         widget_name: &String::from(Page::WatchLater),
//                         child: {
//                             icon_name: Some("alarm-symbolic"),
//                             title: Some(&String::from(Page::WatchLater))
//                         }
//                     },
//                     #[name="filter_page"]
//                     FilterPage(self.model.filters.clone()) {
//                         widget_name: &String::from(Page::Filters),
//                         child: {
//                             icon_name: Some("funnel-symbolic"),
//                             title: Some(&String::from(Page::Filters))
//                         }
//                     },
//                     #[name="subscriptions_page"]
//                     SubscriptionsPage(self.model.subscription_list.clone()) {
//                         widget_name: &String::from(Page::Subscriptions),
//                         child: {
//                             icon_name: Some("library-artists-symbolic"),
//                             title: Some(&String::from(Page::Subscriptions))
//                         }
//                     }
//                 },
//             },
//             delete_event(_, _) => (AppMsg::Quit, Inhibit(false)),
//         }
//     }
// }
