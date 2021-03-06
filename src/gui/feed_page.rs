use crate::gui::app::AppMsg;
use crate::gui::feed_list::{FeedList, FeedListMsg};
use crate::youtube_feed::feed::Feed;

use gtk::prelude::*;
use gtk::Orientation::Vertical;
use relm::StreamHandle;
use relm::Widget;
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum FeedPageMsg {
    Reload,
    SetFeed(Feed),
}

pub struct FeedPageModel {
    app_stream: StreamHandle<AppMsg>,
}

#[widget]
impl Widget for FeedPage {
    fn model(app_stream: StreamHandle<AppMsg>) -> FeedPageModel {
        FeedPageModel { app_stream }
    }

    fn update(&mut self, event: FeedPageMsg) {
        match event {
            FeedPageMsg::Reload => {
                self.model.app_stream.emit(AppMsg::Reload);
            }
            FeedPageMsg::SetFeed(feed) => {
                self.components.feed_list.emit(FeedListMsg::SetFeed(feed));
            }
        }
    }

    view! {
        gtk::Box {
            orientation: Vertical,
            gtk::Button {
                label: "Reload",
                clicked => FeedPageMsg::Reload
            },
            #[name="feed_list"]
            FeedList
        }
    }
}
