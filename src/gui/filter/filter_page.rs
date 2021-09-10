/*
 * Copyright 2021 Julian Schmidhuber <github@schmiddi.anonaddy.com>
 *
 * This file is part of Tubefeeder.
 *
 * Tubefeeder is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Tubefeeder is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Tubefeeder.  If not, see <https://www.gnu.org/licenses/>.
 *
 */
use crate::gui::filter::filter_item::FilterItem;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use gtk::prelude::*;
use gtk::Orientation::Vertical;
use regex::Regex;
use relm::{Channel, ContainerWidget, Relm, Sender, Widget};
use relm_derive::{widget, Msg};
use tf_core::{Observable, Observer};
use tf_filter::{FilterEvent, FilterGroup};
use tf_join::{AnyVideoFilter, Platform};

#[derive(Msg)]
pub enum FilterPageMsg {
    ToggleAddFilter,
    AddFilter,
    NewFilter(AnyVideoFilter),
    RemoveFilter(AnyVideoFilter),
}

pub struct FilterPageModel {
    relm: Relm<FilterPage>,
    add_filter_visible: bool,
    filters: Arc<Mutex<FilterGroup<AnyVideoFilter>>>,
    _filters_observer: Arc<Mutex<Box<dyn Observer<FilterEvent<AnyVideoFilter>> + Send>>>,
    filter_items: HashMap<AnyVideoFilter, relm::Component<FilterItem>>,
}

#[widget]
impl Widget for FilterPage {
    fn model(
        relm: &Relm<Self>,
        filters: Arc<Mutex<FilterGroup<AnyVideoFilter>>>,
    ) -> FilterPageModel {
        let relm_clone = relm.clone();
        let (_channel, sender) = Channel::new(move |msg| relm_clone.stream().emit(msg));

        let observer = Arc::new(Mutex::new(Box::new(FilterPageObserver { sender })
            as Box<dyn Observer<FilterEvent<AnyVideoFilter>> + Send>));

        let filters_clone = filters.clone();
        filters_clone
            .lock()
            .unwrap()
            .iter()
            .for_each(|s| relm.stream().emit(FilterPageMsg::NewFilter(s.clone())));
        filters_clone
            .lock()
            .unwrap()
            .attach(Arc::downgrade(&observer));

        FilterPageModel {
            relm: relm.clone(),
            add_filter_visible: false,
            filters: filters_clone,
            _filters_observer: observer,
            filter_items: HashMap::new(),
        }
    }

    fn update(&mut self, event: FilterPageMsg) {
        match event {
            FilterPageMsg::ToggleAddFilter => {
                self.model.add_filter_visible = !self.model.add_filter_visible;
            }
            FilterPageMsg::AddFilter => self.add_filter(),
            FilterPageMsg::NewFilter(sub) => self.new_filter(sub),
            FilterPageMsg::RemoveFilter(sub) => self.remove_filter(sub),
        }
    }

    fn add_filter(&mut self) {
        let title = self.widgets.title_entry.text();
        let subscription = self.widgets.subscription_entry.text();

        let title_opt = if title.is_empty() { None } else { Some(title) };
        let subscription_opt = if subscription.is_empty() {
            None
        } else {
            Some(subscription)
        };

        let title_regex = title_opt.map(|s| Regex::new(&s));
        let subscription_regex = subscription_opt.map(|s| Regex::new(&s));

        if let Some(Err(_)) = title_regex {
            // TODO: Error Handling
            return;
        }
        if let Some(Err(_)) = subscription_regex {
            // TODO: Error Handling
            return;
        }

        self.widgets.title_entry.set_text("");
        self.widgets.subscription_entry.set_text("");
        self.model
            .relm
            .stream()
            .emit(FilterPageMsg::ToggleAddFilter);

        self.model.filters.lock().unwrap().add(
            AnyVideoFilter::new(
                Some(Platform::Youtube),
                title_regex.map(|r| r.unwrap()),
                subscription_regex.map(|r| r.unwrap()),
            )
            .into(),
        );
    }

    fn new_filter(&mut self, filter: AnyVideoFilter) {
        if self.model.filter_items.get(&filter).is_none() {
            let filter_item = self
                .widgets
                .filter_list
                .add_widget::<FilterItem>((filter.clone(), self.model.filters.clone()));
            self.model.filter_items.insert(filter, filter_item);
        }
    }

    fn remove_filter(&mut self, filter: AnyVideoFilter) {
        if let Some(filter_item) = self.model.filter_items.get(&filter) {
            self.widgets.filter_list.remove(filter_item.widget());
            self.model.filter_items.remove(&filter);
        }
    }

    fn init_view(&mut self) {
        self.widgets.filter_entry_box.hide();
    }

    view! {
        gtk::Box {
            orientation: Vertical,

            #[name="filter_entry_box"]
            gtk::Box {
                visible: self.model.add_filter_visible,
                #[name="title_entry"]
                gtk::Entry {
                    placeholder_text: Some("Title")
                },
                #[name="subscription_entry"]
                gtk::Entry {
                    placeholder_text: Some("Channel name")
                },
                gtk::Button {
                    clicked => FilterPageMsg::AddFilter,
                    image: Some(&gtk::Image::from_icon_name(Some("go-next-symbolic"), gtk::IconSize::LargeToolbar)),
                }
            },

            gtk::ScrolledWindow {
                hexpand: true,
                vexpand: true,
                gtk::Viewport {
                    #[name="filter_list"]
                    gtk::ListBox {
                        selection_mode: gtk::SelectionMode::None

                    }
                }
            }
        }
    }
}
pub struct FilterPageObserver {
    sender: Sender<FilterPageMsg>,
}

impl Observer<FilterEvent<AnyVideoFilter>> for FilterPageObserver {
    fn notify(&mut self, message: FilterEvent<AnyVideoFilter>) {
        match message {
            FilterEvent::Add(filter) => {
                let _ = self.sender.send(FilterPageMsg::NewFilter(filter));
            }
            FilterEvent::Remove(filter) => {
                let _ = self.sender.send(FilterPageMsg::RemoveFilter(filter));
            }
        }
    }
}
