use crate::youtube_feed::feed::Entry;

use relm::Relm;
use relm::Widget;
use relm_derive::widget;

use gtk::prelude::*;
use gtk::Justification;
use gtk::Orientation::Vertical;

use pango::{EllipsizeMode, WrapMode};

#[widget]
impl Widget for FeedListItem {
    fn model(_: &Relm<Self>, entry: Entry) -> Entry {
        entry
    }

    fn update(&mut self, _: ()) {}

    view! {
        gtk::ListBoxRow {
            gtk::Box {
                orientation: Vertical,
                gtk::Label {
                    text: &self.model.title,
                    ellipsize: EllipsizeMode::End,
                    property_wrap: true,
                    property_wrap_mode: WrapMode::Word,
                    lines: 2,
                    justify: Justification::Center

                },
                gtk::Label {
                    text: &self.model.author.name,
                    ellipsize: EllipsizeMode::End,
                    property_wrap: true,
                    property_wrap_mode: WrapMode::Word
                },
                gtk::Label {
                    text: &self.model.published.to_string(),
                    ellipsize: EllipsizeMode::End,
                    property_wrap: true,
                    property_wrap_mode: WrapMode::Word
                }
            }
        }
    }
}
