use gtk::prelude::*;
use gtk::{Builder, Label, ListBoxRow};

pub struct FeedItemBuilder {
    title: String,
    creator: String,
    date: String,
}

impl FeedItemBuilder {
    pub fn new() -> Self {
        FeedItemBuilder {
            title: "".into(),
            creator: "".into(),
            date: "".into(),
        }
    }

    pub fn title(&mut self, title: String) -> &mut Self {
        self.title = title;
        self
    }

    pub fn creator(&mut self, creator: String) -> &mut Self {
        self.creator = creator;
        self
    }

    pub fn date(&mut self, date: String) -> &mut Self {
        self.date = date;
        self
    }

    pub fn build(&mut self) -> ListBoxRow {
        let feed_item_src = include_str!("../../glade/feed_item.glade");
        let builder = Builder::from_string(feed_item_src);

        let feed_item: ListBoxRow = builder
            .get_object("feed_item")
            .expect("could not get feed item");
        let feed_label_title: Label = builder
            .get_object("feed_label_title")
            .expect("could not get feed label title");
        let feed_label_creator: Label = builder
            .get_object("feed_label_creator")
            .expect("could not get feed label creator");
        let feed_label_date: Label = builder
            .get_object("feed_label_date")
            .expect("could not get feed label date");

        feed_label_title.set_text(&self.title);
        feed_label_creator.set_text(&self.creator);
        feed_label_date.set_text(&self.date);

        feed_item
    }
}
