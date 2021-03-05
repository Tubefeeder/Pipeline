use crate::gui::feed_page::{FeedPage, FeedPageMsg};
use crate::gui::subscriptions_page::SubscriptionsPage;
use crate::subscriptions::channel::ChannelGroup;
use crate::youtube_feed::feed::Feed;

use std::path::PathBuf;
use std::thread;

use relm::Relm;
use relm::StreamHandle;
use relm::Widget;
use relm_derive::{widget, Msg};

use gtk::prelude::*;
use gtk::Inhibit;
use gtk::Orientation::Vertical;

use libhandy::ViewSwitcherBarBuilder;

#[derive(Msg)]
pub enum AppMsg {
    Reload,
    Quit,
}

pub struct AppModel {
    app_stream: StreamHandle<AppMsg>,
    _subscriptions_file: PathBuf,
    subscriptions: ChannelGroup,
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

        let subscriptions = ChannelGroup::get_from_file(subscriptions_file_path.clone())
            .expect("could not parse subscriptions file");

        AppModel {
            app_stream: relm.stream().clone(),
            _subscriptions_file: subscriptions_file_path,
            subscriptions,
        }
    }

    fn update(&mut self, event: AppMsg) {
        match event {
            AppMsg::Reload => {
                let stream = self.components.feed_page.stream().clone();
                let (_channel, sender) = relm::Channel::new(move |feed_option: Result<Feed, _>| {
                    stream.emit(FeedPageMsg::SetFeed(feed_option.unwrap_or(Feed::empty())));
                });

                let subscriptions = self.model.subscriptions.clone();

                thread::spawn(move || {
                    sender
                        .send(futures::executor::block_on(subscriptions.get_feed()))
                        .expect("could not send feed");
                });
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
    }

    view! {
        gtk::Window {
            #[name="view_switcher_box"]
            gtk::Box {
                orientation: Vertical,
                #[name="application_stack"]
                gtk::Stack {
                    #[name="feed_page"]
                    FeedPage(self.model.app_stream.clone()) {
                        child: {
                            title: Some("Feed")
                        }
                    },
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
