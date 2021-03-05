use crate::youtube_feed::feed::Entry;

use relm::Relm;
use relm::Widget;
use relm_derive::widget;

use gtk::prelude::*;
use gtk::Justification;
use gtk::Orientation;

use pango::{AttrList, Attribute, EllipsizeMode, WrapMode};

#[widget]
impl Widget for FeedListItem {
    fn model(_: &Relm<Self>, entry: Entry) -> Entry {
        entry
    }

    fn update(&mut self, _: ()) {}

    fn init_view(&mut self) {
        let title_attr_list = AttrList::new();
        title_attr_list.insert(Attribute::new_size(17 * pango::SCALE).unwrap());
        self.widgets
            .label_title
            .set_attributes(Some(&title_attr_list));

        let author_attr_list = AttrList::new();
        author_attr_list.insert(Attribute::new_size(10 * pango::SCALE).unwrap());
        self.widgets
            .label_author
            .set_attributes(Some(&author_attr_list));

        let date_attr_list = author_attr_list.clone();
        self.widgets
            .label_date
            .set_attributes(Some(&date_attr_list));
    }

    view! {
        gtk::ListBoxRow {
            gtk::Box {
                orientation: Orientation::Vertical,
                #[name="label_title"]
                gtk::Label {
                    text: &self.model.title,
                    ellipsize: EllipsizeMode::End,
                    property_wrap: true,
                    property_wrap_mode: WrapMode::Word,
                    lines: 2,
                    justify: Justification::Center
                },
                #[name="label_author"]
                gtk::Label {
                    text: &self.model.author.name,
                    ellipsize: EllipsizeMode::End,
                    property_wrap: true,
                    property_wrap_mode: WrapMode::Word
                },
                #[name="label_date"]
                gtk::Label {
                    text: &self.model.published.to_string(),
                    ellipsize: EllipsizeMode::End,
                    property_wrap: true,
                    property_wrap_mode: WrapMode::Word
                },
            }
        }
    }
}
