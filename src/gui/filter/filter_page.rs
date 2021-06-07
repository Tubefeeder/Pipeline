use crate::filter::{EntryFilter, EntryFilterGroup};
use crate::gui::app::AppMsg;
use crate::gui::filter::filter_item::FilterItem;
use crate::gui::lazy_list::{LazyList, LazyListMsg, ListElementBuilder};

use gtk::prelude::*;
use gtk::Orientation::Vertical;
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

pub struct FilterElementBuilder {
    chunks: Vec<Vec<(EntryFilter, StreamHandle<AppMsg>)>>,
}

impl FilterElementBuilder {
    fn new(group: EntryFilterGroup, app_stream: StreamHandle<AppMsg>) -> Self {
        FilterElementBuilder {
            chunks: group
                .get_filters()
                .chunks(20)
                .map(|slice| {
                    slice
                        .iter()
                        .map(|c| (c.clone(), app_stream.clone()))
                        .collect()
                })
                .collect::<Vec<Vec<(EntryFilter, StreamHandle<AppMsg>)>>>(),
        }
    }
}

impl ListElementBuilder<FilterItem> for FilterElementBuilder {
    fn poll(&mut self) -> Vec<(EntryFilter, StreamHandle<AppMsg>)> {
        if !self.chunks.is_empty() {
            self.chunks.remove(0)
        } else {
            vec![]
        }
    }
}

#[derive(Msg)]
pub enum FilterPageMsg {
    SetFilters(EntryFilterGroup),
    ToggleAddFilter,
    AddFilter,
}

pub struct FilterPageModel {
    relm: Relm<FilterPage>,
    app_stream: StreamHandle<AppMsg>,
    add_filter_visible: bool,
}

#[widget]
impl Widget for FilterPage {
    fn model(relm: &Relm<Self>, app_stream: StreamHandle<AppMsg>) -> FilterPageModel {
        FilterPageModel {
            relm: relm.clone(),
            app_stream,
            add_filter_visible: false,
        }
    }

    fn update(&mut self, event: FilterPageMsg) {
        match event {
            FilterPageMsg::SetFilters(filter_group) => {
                self.components
                    .filter_list
                    .emit(LazyListMsg::SetListElementBuilder(Box::new(
                        FilterElementBuilder::new(filter_group, self.model.app_stream.clone()),
                    )));
            }
            FilterPageMsg::ToggleAddFilter => {
                self.model.add_filter_visible = !self.model.add_filter_visible;
            }
            FilterPageMsg::AddFilter => {
                let filter_title = &self.widgets.filter_title_entry.get_text();
                let filter_channel = &self.widgets.filter_channel_entry.get_text();

                self.widgets.filter_title_entry.set_text("");
                self.widgets.filter_channel_entry.set_text("");
                self.model
                    .relm
                    .stream()
                    .emit(FilterPageMsg::ToggleAddFilter);

                let new_filter = EntryFilter::new(filter_title, filter_channel);

                if let Ok(filter) = new_filter {
                    self.model.app_stream.emit(AppMsg::AddFilter(filter));
                } else {
                    self.model
                        .app_stream
                        .emit(AppMsg::Error(new_filter.err().unwrap()));
                }
            }
        }
    }

    view! {
        gtk::Box {
            orientation: Vertical,

            gtk::Box {
                visible: self.model.add_filter_visible,
                #[name="filter_title_entry"]
                gtk::Entry {
                    placeholder_text: Some("Title")
                },
                #[name="filter_channel_entry"]
                gtk::Entry {
                    placeholder_text: Some("Channel")
                },
                gtk::Button {
                    clicked => FilterPageMsg::AddFilter,
                    image: Some(&gtk::Image::from_icon_name(Some("go-next-symbolic"), gtk::IconSize::LargeToolbar)),
                }
            },

            #[name="filter_list"]
            LazyList<FilterItem>
        }
    }
}
