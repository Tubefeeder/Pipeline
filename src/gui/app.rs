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
use crate::gui::error_label::ErrorLabel;
use crate::gui::feed::{FeedPage, FeedPageMsg};
use crate::gui::filter::{FilterPage, FilterPageMsg};
use crate::gui::header_bar::{HeaderBar, HeaderBarMsg, Page};
use crate::gui::playlist::PlaylistPage;
use crate::gui::subscriptions::{SubscriptionsPage, SubscriptionsPageMsg};

use tf_core::{ErrorStore, Generator};
use tf_filter::{FilterEvent, FilterGroup};
use tf_join::{AnySubscriptionList, AnyVideo, AnyVideoFilter, Joiner, SubscriptionEvent};
use tf_observer::{Observable, Observer};
use tf_playlist::{PlaylistEvent, PlaylistManager};

use std::str::FromStr;
use std::sync::{Arc, Mutex};

use gtk::prelude::*;
use gtk::traits::SettingsExt;
use gtk::{Inhibit, Orientation::Vertical};
use libhandy::ViewSwitcherBarBuilder;
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

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

pub fn init_icons() {
    let res_bytes = include_bytes!("../../resources.gresource");

    let gbytes = glib::Bytes::from_static(res_bytes.as_ref());
    let resource = gio::Resource::from_data(&gbytes).unwrap();

    let icon_theme = gtk::IconTheme::default().unwrap_or_default();

    icon_theme.add_resource_path("/");
    icon_theme.add_resource_path("/org/gnome/design/IconLibrary/data/icons/");

    gio::resources_register(&resource);
}

#[derive(Msg)]
pub enum AppMsg {
    Loading(bool),
    Reload,
    ToggleAddSubscription,
    ToggleAddFilter,
    Quit,
}

pub struct AppModel {
    joiner: Joiner,
    playlist_manager: PlaylistManager<String, AnyVideo>,
    errors: ErrorStore,
    app_stream: StreamHandle<AppMsg>,

    _subscription_file_manager: Arc<Mutex<Box<dyn Observer<SubscriptionEvent> + Send>>>,
    _filter_file_manager: Arc<Mutex<Box<dyn Observer<FilterEvent<AnyVideoFilter>> + Send>>>,
    _watchlater_file_manager: Arc<Mutex<Box<dyn Observer<PlaylistEvent<AnyVideo>> + Send>>>,
    subscription_list: AnySubscriptionList,
    filters: Arc<Mutex<FilterGroup<AnyVideoFilter>>>,
    loading: bool,
}

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        self.widgets.window.resize(800, 500);

        // Build view switcher
        let view_switcher = ViewSwitcherBarBuilder::new()
            .stack(&self.widgets.application_stack)
            .reveal(true)
            .build();

        self.widgets.view_switcher_box.add(&view_switcher);
        view_switcher.show();

        // Build header bar
        let header_bar_stream = self.components.header_bar.stream();
        header_bar_stream.emit(HeaderBarMsg::SetPage(Page::Feed));

        self.widgets
            .application_stack
            .connect_visible_child_notify(move |stack| {
                let child = stack.visible_child().unwrap();
                let title = child.widget_name();
                header_bar_stream.emit(HeaderBarMsg::SetPage(Page::from_str(&title).unwrap()));
            });

        self.widgets.loading_spinner.start();

        self.model.app_stream.emit(AppMsg::Reload);
    }

    fn model(relm: &Relm<Self>, joiner: Joiner) -> AppModel {
        init_icons();

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

        let mut filter_file_path = user_data_dir.clone();
        filter_file_path.push("filters.csv");

        let mut watchlater_file_path = user_data_dir.clone();
        watchlater_file_path.push("playlist_watch_later.csv");

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
            app_stream: relm.stream().clone(),
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

    fn update(&mut self, event: AppMsg) {
        match event {
            AppMsg::Loading(loading) => {
                self.model.loading = loading;
            }
            AppMsg::Reload => {
                self.reload();
            }
            AppMsg::ToggleAddSubscription => {
                self.components
                    .subscriptions_page
                    .emit(SubscriptionsPageMsg::ToggleAddSubscription);
            }
            AppMsg::ToggleAddFilter => {
                self.components
                    .filter_page
                    .emit(FilterPageMsg::ToggleAddFilter);
            }
            AppMsg::Quit => {
                gtk::main_quit();

                let mut user_cache_dir = glib::user_cache_dir();
                user_cache_dir.push("tubefeeder");

                if user_cache_dir.exists() {
                    std::fs::remove_dir_all(user_cache_dir).unwrap_or(());
                }
            }
        }
    }

    fn reload(&mut self) {
        let loading_spinner = self.widgets.loading_spinner.clone();
        loading_spinner.set_visible(true);

        let feed_stream = self.components.feed_page.stream();
        let app_stream = self.model.app_stream.clone();
        app_stream.emit(AppMsg::Loading(true));

        let (_channel, sender) = relm::Channel::new(move |feed: std::vec::IntoIter<AnyVideo>| {
            feed_stream.emit(FeedPageMsg::SetFeed(Box::new(feed)));
            app_stream.emit(AppMsg::Loading(false));
        });

        let joiner = self.model.joiner.clone();
        let errors = self.model.errors.clone();
        errors.clear();
        tokio::spawn(async move {
            let feed = joiner.generate(&errors).await;
            sender.send(feed).unwrap()
        });
    }

    view! {
        #[name="window"]
        gtk::Window {
            titlebar: view! {
                #[name="header_bar"]
                HeaderBar(self.model.app_stream.clone()) {
                }
            },
            #[name="view_switcher_box"]
            gtk::Box {
                gtk::Box {
                    orientation: Vertical,
                    #[name="error_label"]
                    ErrorLabel(self.model.errors.clone()) {},
                    #[name="loading_spinner"]
                    gtk::Spinner {
                        visible: self.model.loading,
                        active: true
                    }
                },
                orientation: Vertical,
                #[name="application_stack"]
                gtk::Stack {
                    #[name="feed_page"]
                    FeedPage(self.model.playlist_manager.clone()) {
                        widget_name: &String::from(Page::Feed),
                        child: {
                            icon_name: Some("go-home-symbolic"),
                            title: Some(&String::from(Page::Feed))
                        }
                    },
                    #[name="watch_later_page"]
                    PlaylistPage(self.model.playlist_manager.clone(), "WATCHLATER".to_string()) {
                        widget_name: &String::from(Page::WatchLater),
                        child: {
                            icon_name: Some("alarm-symbolic"),
                            title: Some(&String::from(Page::WatchLater))
                        }
                    },
                    #[name="filter_page"]
                    FilterPage(self.model.filters.clone()) {
                        widget_name: &String::from(Page::Filters),
                        child: {
                            icon_name: Some("funnel-symbolic"),
                            title: Some(&String::from(Page::Filters))
                        }
                    },
                    #[name="subscriptions_page"]
                    SubscriptionsPage(self.model.subscription_list.clone()) {
                        widget_name: &String::from(Page::Subscriptions),
                        child: {
                            icon_name: Some("library-artists-symbolic"),
                            title: Some(&String::from(Page::Subscriptions))
                        }
                    }
                },
            },
            delete_event(_, _) => (AppMsg::Quit, Inhibit(false)),
        }
    }
}
