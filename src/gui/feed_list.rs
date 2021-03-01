use crate::gui::feed_item::FeedItemBuilder;
use crate::youtube_feed::feed::Feed;

use std::process::{Command, Stdio};

use gtk::prelude::*;
use gtk::{ListBox, ListBoxRow, SelectionMode};

#[derive(Clone)]
pub struct FeedList {
    feed_list: ListBox,
    feed: Option<Feed>,
}

impl FeedList {
    pub fn new(feed_list: ListBox) -> Self {
        feed_list.set_selection_mode(SelectionMode::None);

        FeedList {
            feed_list,
            feed: None,
        }
    }

    pub fn set_feed(&mut self, feed: Feed) {
        self.feed_list.foreach(|w| self.feed_list.remove(w));
        self.feed = Some(feed.clone());
        for entry in &feed.entries {
            let new_elem: ListBoxRow = FeedItemBuilder::new()
                .title(entry.title.clone())
                .creator(entry.author.name.clone())
                .date(entry.updated.to_string().clone())
                .build();

            self.feed_list.add(&new_elem);
        }

        self.feed_list.connect_row_activated(move |l, w| {
            let index = l.get_children().iter().position(|r| &w == &r);

            let _res = Command::new("mpv")
                .arg(&feed.entries[index.unwrap()].link.href)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
        });
    }
}
