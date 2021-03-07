use crate::gui::feed_page::{FeedPage, FeedPageMsg};
use crate::gui::subscriptions_page::{SubscriptionsPage, SubscriptionsPageMsg};
use crate::subscriptions::channel::{Channel, ChannelGroup};
use crate::youtube_feed::feed::Feed;

use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;

use gtk::prelude::*;
use gtk::{Inhibit, Justification, Orientation::Vertical};
use libhandy::ViewSwitcherBarBuilder;
use pango::{EllipsizeMode, WrapMode};
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum AppMsg {
    Error(String),
    Loading(bool),
    Reload,
    SetSubscriptions(ChannelGroup),
    Quit,
}

pub struct AppModel {
    app_stream: StreamHandle<AppMsg>,
    subscriptions_file: PathBuf,
    subscriptions: ChannelGroup,
    error_msg: String,
    loading: bool,
}

impl AppModel {
    fn reload_subscriptions(&mut self) {
        self.subscriptions = ChannelGroup::get_from_file(self.subscriptions_file.clone())
            .unwrap_or(ChannelGroup::new());
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

        let mut model = AppModel {
            app_stream: relm.stream().clone(),
            subscriptions_file: subscriptions_file_path,
            subscriptions: ChannelGroup::new(),
            error_msg: "".to_string(),
            loading: false,
        };

        model.reload_subscriptions();

        model
    }

    fn update(&mut self, event: AppMsg) {
        match event {
            AppMsg::Error(msg) => {
                self.model.error_msg = msg;
            }
            AppMsg::Loading(loading) => {
                self.model.loading = loading;
            }
            AppMsg::Reload => {
                let loading_spinner = self.widgets.loading_spinner.clone();
                loading_spinner.set_visible(true);

                let feed_stream = self.components.feed_page.stream().clone();
                let app_stream = self.model.app_stream.clone();
                let subscriptions1 = self.model.subscriptions.clone();

                app_stream.emit(AppMsg::Error("".to_string()));
                app_stream.emit(AppMsg::Loading(true));

                let (_channel, sender) = relm::Channel::new(move |feed_option: Result<Feed, _>| {
                    if let Err(e) = feed_option.clone() {
                        app_stream.emit(AppMsg::Error(format!("{}", e)));
                    }

                    feed_stream.emit(FeedPageMsg::SetFeed(
                        feed_option.clone().unwrap_or(Feed::empty()),
                    ));

                    if let Ok(feed) = feed_option {
                        let channels: HashMap<String, String> = feed
                            .entries
                            .iter()
                            .map(|e| {
                                let channel: Channel = e.author.clone().into();
                                (channel.get_id(), channel.get_name().unwrap())
                            })
                            .collect();

                        let result = subscriptions1
                            .channels
                            .iter()
                            .map(|channel| {
                                let mut result = channel.clone();
                                if let Some(name) = channels.get(&result.get_id()) {
                                    result.name = Some(name.to_string());
                                }
                                result
                            })
                            .collect();

                        app_stream
                            .emit(AppMsg::SetSubscriptions(ChannelGroup { channels: result }));
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
            AppMsg::SetSubscriptions(subscriptions) => {
                self.model.subscriptions = subscriptions;
                self.components
                    .subscriptions_page
                    .emit(SubscriptionsPageMsg::SetSubscriptions(
                        self.model.subscriptions.clone(),
                    ));
            }
            AppMsg::Quit => gtk::main_quit(),
        }
    }

    fn init_view(&mut self) {
        let view_switcher = ViewSwitcherBarBuilder::new()
            .stack(&self.widgets.application_stack)
            .reveal(true)
            .build();
        self.widgets.view_switcher_box.add(&view_switcher);
        self.widgets.view_switcher_box.show_all();

        self.widgets.loading_spinner.start();

        self.model.app_stream.emit(AppMsg::Reload);
    }

    view! {
        gtk::Window {
            #[name="view_switcher_box"]
            gtk::Box {
                gtk::Box {
                    orientation: Vertical,
                    #[name="error_label"]
                    gtk::Label {
                        visible: !self.model.error_msg.is_empty(),
                        ellipsize: EllipsizeMode::End,
                        property_wrap: true,
                        property_wrap_mode: WrapMode::Word,
                        lines: 2,
                        justify: Justification::Center,
                        text: &self.model.error_msg
                    },
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
                        child: {
                            title: Some("Feed")
                        }
                    },
                    #[name="subscriptions_page"]
                    SubscriptionsPage {
                        child: {
                            title: Some("Subscriptions")
                        }
                    }
                },
            },
            delete_event(_, _) => (AppMsg::Quit, Inhibit(false)),
        }
    }
}
