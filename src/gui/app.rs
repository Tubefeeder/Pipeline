use crate::errors::Error;
use crate::filter::{EntryFilter, EntryFilterGroup};
use crate::gui::error_label::{ErrorLabel, ErrorLabelMsg};
use crate::gui::feed::{FeedPage, FeedPageMsg};
use crate::gui::filter::{FilterPage, FilterPageMsg};
use crate::gui::header_bar::{HeaderBar, HeaderBarMsg, Page};
use crate::gui::subscriptions::{SubscriptionsPage, SubscriptionsPageMsg};
use crate::subscriptions::{Channel, ChannelGroup};
use crate::youtube_feed::{Entry, Feed};

use std::path::PathBuf;
use std::str::FromStr;
use std::thread;

use gtk::prelude::*;
use gtk::{Inhibit, Orientation::Vertical};
use libhandy::ViewSwitcherBarBuilder;
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum AppMsg {
    Loading(bool),
    Reload,
    SetSubscriptions(ChannelGroup),
    AddSubscription(Channel),
    RemoveSubscription(Channel),
    ToggleAddSubscription,
    SetFilters(EntryFilterGroup),
    AddFilter(EntryFilter),
    RemoveFilter(EntryFilter),
    ToggleAddFilter,
    Error(Error),
    ToggleWatchLater(Entry),
    Quit,
}

pub struct AppModel {
    app_stream: StreamHandle<AppMsg>,

    subscriptions_file: PathBuf,
    subscriptions: ChannelGroup,

    filter_file: PathBuf,
    filter: EntryFilterGroup,

    watch_later: Feed,

    loading: bool,
    startup_err: Option<Error>,
}

impl AppModel {
    fn reload_subscriptions(&mut self) -> Result<(), Error> {
        let subscription_res = ChannelGroup::get_from_path(&self.subscriptions_file);
        self.subscriptions = subscription_res.clone().unwrap_or(ChannelGroup::new());

        if let Err(e) = subscription_res {
            Err(e)
        } else {
            Ok(())
        }
    }

    fn reload_filters(&mut self) -> Result<(), Error> {
        let filter_res = EntryFilterGroup::get_from_path(&self.filter_file);
        self.filter = filter_res.clone().unwrap_or(EntryFilterGroup::new());

        if let Err(e) = filter_res {
            Err(e)
        } else {
            Ok(())
        }
    }
}

#[widget]
impl Widget for Win {
    fn model(relm: &Relm<Self>, _: ()) -> AppModel {
        let mut user_data_dir =
            glib::get_user_data_dir().expect("could not get user data directory");
        user_data_dir.push("tubefeeder");

        if !user_data_dir.exists() {
            std::fs::create_dir_all(user_data_dir.clone()).expect("could not create user data dir");
        }

        let mut subscriptions_file_path = user_data_dir.clone();
        subscriptions_file_path.push("subscriptions.db");

        let mut filter_file_path = user_data_dir.clone();
        filter_file_path.push("filters.db");

        let mut model = AppModel {
            app_stream: relm.stream().clone(),
            subscriptions_file: subscriptions_file_path,
            subscriptions: ChannelGroup::new(),
            filter_file: filter_file_path,
            filter: EntryFilterGroup::new(),
            watch_later: Feed::empty(),
            loading: false,
            startup_err: None,
        };

        let err = model.reload_subscriptions();
        let err2 = model.reload_filters();

        if let Err(e) = err {
            model.startup_err = Some(e);
        } else if let Err(e) = err2 {
            model.startup_err = Some(e)
        }

        model
    }

    fn update(&mut self, event: AppMsg) {
        match event {
            AppMsg::Loading(loading) => {
                self.model.loading = loading;
            }
            AppMsg::Reload => {
                self.reload();
            }
            AppMsg::SetSubscriptions(subscriptions) => {
                self.model.subscriptions = subscriptions;
                self.components
                    .subscriptions_page
                    .emit(SubscriptionsPageMsg::SetSubscriptions(
                        self.model.subscriptions.clone(),
                    ));
            }
            AppMsg::AddSubscription(channel) => {
                let mut new_group = self.model.subscriptions.clone();
                new_group.add(channel);
                let write_res = new_group.write_to_path(&self.model.subscriptions_file);

                if let Err(e) = write_res {
                    self.components
                        .error_label
                        .emit(ErrorLabelMsg::Set(Some(e)));
                } else {
                    self.model
                        .app_stream
                        .emit(AppMsg::SetSubscriptions(new_group));
                }
            }
            AppMsg::RemoveSubscription(channel) => {
                let mut new_group = self.model.subscriptions.clone();
                new_group.remove(channel);
                let write_res = new_group.write_to_path(&self.model.subscriptions_file);

                if let Err(e) = write_res {
                    self.components
                        .error_label
                        .emit(ErrorLabelMsg::Set(Some(e)));
                } else {
                    self.model
                        .app_stream
                        .emit(AppMsg::SetSubscriptions(new_group));
                }
            }
            AppMsg::ToggleAddSubscription => {
                self.components
                    .subscriptions_page
                    .emit(SubscriptionsPageMsg::ToggleAddSubscription);
            }
            AppMsg::SetFilters(filter) => {
                self.model.filter = filter;
                self.components
                    .filter_page
                    .emit(FilterPageMsg::SetFilters(self.model.filter.clone()));
            }
            AppMsg::AddFilter(filter) => {
                let mut new_filter_group = self.model.filter.clone();
                new_filter_group.add(filter);
                let write_res = new_filter_group.write_to_path(&self.model.filter_file);

                if let Err(e) = write_res {
                    self.components
                        .error_label
                        .emit(ErrorLabelMsg::Set(Some(e)));
                } else {
                    self.model
                        .app_stream
                        .emit(AppMsg::SetFilters(new_filter_group));
                }
            }
            AppMsg::RemoveFilter(filter) => {
                let mut new_filter_group = self.model.filter.clone();
                new_filter_group.remove(filter);
                let write_res = new_filter_group.write_to_path(&self.model.filter_file);

                if let Err(e) = write_res {
                    self.components
                        .error_label
                        .emit(ErrorLabelMsg::Set(Some(e)));
                } else {
                    self.model
                        .app_stream
                        .emit(AppMsg::SetFilters(new_filter_group));
                }
            }
            AppMsg::ToggleAddFilter => {
                self.components
                    .filter_page
                    .emit(FilterPageMsg::ToggleAddFilter);
            }
            AppMsg::ToggleWatchLater(entry) => {
                let current = &mut self.model.watch_later.entries;
                if !current.contains(&entry) {
                    current.push(entry);
                } else {
                    current.retain(|e| e != &entry);
                }

                self.components
                    .watch_later_page
                    .emit(FeedPageMsg::SetFeed(self.model.watch_later.clone()));
            }
            AppMsg::Error(error) => {
                self.components
                    .error_label
                    .emit(ErrorLabelMsg::Set(Some(error)));
            }
            AppMsg::Quit => gtk::main_quit(),
        }
    }

    fn reload(&mut self) {
        let loading_spinner = self.widgets.loading_spinner.clone();
        loading_spinner.set_visible(true);

        let feed_stream = self.components.feed_page.stream().clone();
        let app_stream = self.model.app_stream.clone();
        let mut subscriptions1 = self.model.subscriptions.clone();
        let error_label_stream = self.components.error_label.stream().clone();

        let filter = self.model.filter.clone();

        // Dont override errors from startup
        if self.model.startup_err.is_none() {
            error_label_stream.emit(ErrorLabelMsg::Set(None));
        } else {
            self.model.startup_err = None;
        }

        app_stream.emit(AppMsg::Loading(true));

        let (_channel, sender) = relm::Channel::new(move |feed_option: Result<Feed, _>| {
            if let Err(e) = feed_option.clone() {
                error_label_stream.emit(ErrorLabelMsg::Set(Some(e)));
            }

            let mut feed = feed_option.clone().unwrap_or(Feed::empty());
            feed.filter(&filter);

            feed_stream.emit(FeedPageMsg::SetFeed(feed));

            if let Ok(feed) = feed_option {
                let channels = feed.extract_channels();
                subscriptions1.resolve_name(&channels);

                app_stream.emit(AppMsg::SetSubscriptions(subscriptions1.clone()));
            }
            app_stream.emit(AppMsg::Loading(false));
        });

        let subscriptions2 = self.model.subscriptions.clone();

        thread::spawn(move || {
            sender
                .send(futures::executor::block_on(subscriptions2.get_feed()))
                .expect("could not send feed");
        });
    }

    fn init_view(&mut self) {
        // Build view switcher
        let view_switcher = ViewSwitcherBarBuilder::new()
            .stack(&self.widgets.application_stack)
            .reveal(true)
            .build();

        self.widgets.view_switcher_box.add(&view_switcher);
        self.widgets.view_switcher_box.show_all();

        // Build header bar
        let header_bar_stream = self.components.header_bar.stream().clone();
        header_bar_stream.emit(HeaderBarMsg::SetPage(Page::Feed));

        self.widgets
            .application_stack
            .connect_property_visible_child_notify(move |stack| {
                let child = stack.get_visible_child().unwrap();
                let title = child.get_widget_name();
                header_bar_stream.emit(HeaderBarMsg::SetPage(Page::from_str(&title).unwrap()));
            });

        self.widgets.loading_spinner.start();

        // Hide the subscription entry (Visible by default, no idea why).
        let subscriptions_page = &self.components.subscriptions_page;
        subscriptions_page.emit(SubscriptionsPageMsg::ToggleAddSubscription);
        subscriptions_page.emit(SubscriptionsPageMsg::ToggleAddSubscription);

        // Hide the filter entry (Visible by default, no idea why).
        let filter_page = &self.components.filter_page;
        filter_page.emit(FilterPageMsg::ToggleAddFilter);
        filter_page.emit(FilterPageMsg::ToggleAddFilter);

        self.components
            .error_label
            .emit(ErrorLabelMsg::Set(self.model.startup_err.clone()));

        self.model
            .app_stream
            .emit(AppMsg::SetFilters(self.model.filter.clone()));
        self.model.app_stream.emit(AppMsg::Reload);
    }

    view! {
        gtk::Window {
            decorated: false,
            #[name="view_switcher_box"]
            gtk::Box {
                #[name="header_bar"]
                HeaderBar(self.model.app_stream.clone()) {
                },

                gtk::Box {
                    orientation: Vertical,
                    #[name="error_label"]
                    ErrorLabel {},
                    #[name="loading_spinner"]
                    gtk::Spinner {
                        visible: self.model.loading,
                        property_active: true
                    }
                },
                orientation: Vertical,
                #[name="application_stack"]
                gtk::Stack {
                    #[name="feed_page"]
                    FeedPage(self.model.app_stream.clone()) {
                        widget_name: &String::from(Page::Feed),
                        child: {
                            title: Some(&String::from(Page::Feed))
                        }
                    },
                    #[name="watch_later_page"]
                    FeedPage(self.model.app_stream.clone()) {
                        widget_name: &String::from(Page::WatchLater),
                        child: {
                            title: Some(&String::from(Page::WatchLater))
                        }
                    },
                    #[name="filter_page"]
                    FilterPage(self.model.app_stream.clone()) {
                        widget_name: &String::from(Page::Filters),
                        child: {
                            title: Some(&String::from(Page::Filters))
                        }
                    },
                    #[name="subscriptions_page"]
                    SubscriptionsPage(self.model.app_stream.clone()) {
                        widget_name: &String::from(Page::Subscriptions),
                        child: {
                            title: Some(&String::from(Page::Subscriptions))
                        }
                    }
                },
            },
            delete_event(_, _) => (AppMsg::Quit, Inhibit(false)),
        }
    }
}
