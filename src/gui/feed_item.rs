use crate::gui::thumbnail::{Thumbnail, ThumbnailMsg};
use crate::youtube_feed::feed::Entry;

use gtk::prelude::*;
use gtk::Justification;
use gtk::Orientation;
use pango::{AttrList, Attribute, EllipsizeMode, WrapMode};
use relm::Relm;
use relm::Widget;
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum FeedListItemMsg {
    SetImage,
}

#[widget]
impl Widget for FeedListItem {
    fn model(_: &Relm<Self>, entry: Entry) -> Entry {
        entry
    }

    fn update(&mut self, event: FeedListItemMsg) {
        match event {
            FeedListItemMsg::SetImage => {
                self.components.thumbnail.emit(ThumbnailMsg::SetImage);
            }
        }
    }

    fn init_view(&mut self) {
        let title_attr_list = AttrList::new();
        title_attr_list.insert(Attribute::new_size(12 * pango::SCALE).unwrap());
        self.widgets
            .label_title
            .set_attributes(Some(&title_attr_list));

        let author_attr_list = AttrList::new();
        author_attr_list.insert(Attribute::new_size(8 * pango::SCALE).unwrap());
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
                orientation: Orientation::Horizontal,

                #[name="thumbnail"]
                Thumbnail(self.model.media.thumbnail.clone()),

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
}
