use crate::gui::app::AppMsg;
use crate::gui::feed_item::{FeedListItem, FeedListItemMsg};
use crate::gui::lazy_list::{LazyList, LazyListMsg, ListElementBuilder};
use crate::youtube_feed::{Entry, Feed};

use gtk::prelude::*;
use gtk::Orientation::Vertical;
use relm::{StreamHandle, Widget};
use relm_derive::{widget, Msg};

pub struct FeedElementBuilder {
    chunks: Vec<Vec<Entry>>,
}

impl FeedElementBuilder {
    fn new(feed: Feed) -> Self {
        FeedElementBuilder {
            chunks: feed
                .entries
                .chunks(10)
                .map(|slice| slice.to_vec())
                .collect::<Vec<Vec<Entry>>>(),
        }
    }
}

impl ListElementBuilder<FeedListItem> for FeedElementBuilder {
    fn poll(&mut self) -> Vec<Entry> {
        if !self.chunks.is_empty() {
            self.chunks.remove(0)
        } else {
            vec![]
        }
    }

    fn add_stream(&mut self, stream: StreamHandle<FeedListItemMsg>) {
        stream.emit(FeedListItemMsg::SetImage);
    }

    fn get_clicked_signal(&self) -> Option<FeedListItemMsg> {
        Some(FeedListItemMsg::Clicked)
    }
}

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
                self.components
                    .feed_list
                    .emit(LazyListMsg::SetListElementBuilder(Box::new(
                        FeedElementBuilder::new(feed),
                    )));
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
            LazyList<FeedListItem>
        }
    }
}
