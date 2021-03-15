use crate::filter::EntryFilter;
use crate::gui::app::AppMsg;

use gtk::prelude::*;
use gtk::Align;
use gtk::Orientation::Vertical;
use pango::{AttrList, Attribute, EllipsizeMode};
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum FilterItemMsg {
    Remove,
}

pub struct FilterItemModel {
    filter: EntryFilter,
    app_stream: StreamHandle<AppMsg>,
}

#[widget]
impl Widget for FilterItem {
    fn model(
        _: &Relm<Self>,
        (filter, app_stream): (EntryFilter, StreamHandle<AppMsg>),
    ) -> FilterItemModel {
        FilterItemModel { filter, app_stream }
    }

    fn update(&mut self, event: FilterItemMsg) {
        match event {
            FilterItemMsg::Remove => {
                self.model
                    .app_stream
                    .emit(AppMsg::RemoveFilter(self.model.filter.clone()));
            }
        }
    }

    fn init_view(&mut self) {
        let title_attr_list = AttrList::new();
        title_attr_list.insert(Attribute::new_size(15 * pango::SCALE).unwrap());
        self.widgets
            .label_title
            .set_attributes(Some(&title_attr_list));

        let channel_attr_list = AttrList::new();
        channel_attr_list.insert(Attribute::new_size(12 * pango::SCALE).unwrap());
        self.widgets
            .label_channel
            .set_attributes(Some(&channel_attr_list));
    }

    view! {
        gtk::ListBoxRow {
            gtk::Box {
                gtk::Button {
                    image: Some(&gtk::Image::from_icon_name(Some("list-remove"), gtk::IconSize::LargeToolbar)),
                    clicked => FilterItemMsg::Remove,
                },
                gtk::Box {
                    orientation: Vertical,
                    #[name="label_title"]
                    gtk::Label {
                        text: &self.model.filter.get_title_filter_string(),
                        ellipsize: EllipsizeMode::End,
                        halign: Align::Start
                    },
                    #[name="label_channel"]
                    gtk::Label {
                        text: &self.model.filter.get_channel_filter_string(),
                        ellipsize: EllipsizeMode::End,
                        halign: Align::Start
                    },
                }
            }
        }
    }
}
