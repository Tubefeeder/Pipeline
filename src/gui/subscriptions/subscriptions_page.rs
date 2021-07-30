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
 * along with Foobar.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::errors::Error;
use crate::gui::app::AppMsg;
use crate::gui::lazy_list::{LazyList, LazyListMsg, ListElementBuilder};
use crate::gui::subscriptions::subscription_item::SubscriptionItem;
use crate::subscriptions::{Channel, ChannelGroup};

use std::thread;

use gtk::prelude::*;
use gtk::Orientation::Vertical;
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

pub struct SubscriptionElementBuilder {
    chunks: Vec<Vec<(Channel, StreamHandle<AppMsg>)>>,
}

impl SubscriptionElementBuilder {
    fn new(group: ChannelGroup, app_stream: StreamHandle<AppMsg>) -> Self {
        SubscriptionElementBuilder {
            chunks: group
                .channels
                .chunks(20)
                .map(|slice| {
                    slice
                        .iter()
                        .map(|c| (c.clone(), app_stream.clone()))
                        .collect()
                })
                .collect::<Vec<Vec<(Channel, StreamHandle<AppMsg>)>>>(),
        }
    }
}

impl ListElementBuilder<SubscriptionItem> for SubscriptionElementBuilder {
    fn poll(&mut self) -> Vec<(Channel, StreamHandle<AppMsg>)> {
        if !self.chunks.is_empty() {
            self.chunks.remove(0)
        } else {
            vec![]
        }
    }
}

#[derive(Msg)]
pub enum SubscriptionsPageMsg {
    SetSubscriptions(ChannelGroup),
    ToggleAddSubscription,
    AddSubscription,
}

pub struct SubscriptionsPageModel {
    relm: Relm<SubscriptionsPage>,
    app_stream: StreamHandle<AppMsg>,
    add_subscription_visible: bool,
}

#[widget]
impl Widget for SubscriptionsPage {
    fn model(relm: &Relm<Self>, app_stream: StreamHandle<AppMsg>) -> SubscriptionsPageModel {
        SubscriptionsPageModel {
            relm: relm.clone(),
            app_stream,
            add_subscription_visible: false,
        }
    }

    fn update(&mut self, event: SubscriptionsPageMsg) {
        match event {
            SubscriptionsPageMsg::SetSubscriptions(channels) => {
                self.components
                    .subscription_list
                    .emit(LazyListMsg::SetListElementBuilder(Box::new(
                        SubscriptionElementBuilder::new(channels, self.model.app_stream.clone()),
                    )));
            }
            SubscriptionsPageMsg::ToggleAddSubscription => {
                self.model.add_subscription_visible = !self.model.add_subscription_visible;
            }
            SubscriptionsPageMsg::AddSubscription => self.add_subscription(),
        }
    }

    fn add_subscription(&mut self) {
        let channel_name = self.widgets.channel_name_entry.get_text();

        self.widgets.channel_name_entry.set_text("");
        self.model
            .relm
            .stream()
            .emit(SubscriptionsPageMsg::ToggleAddSubscription);

        let app_stream = self.model.app_stream.clone();

        let (_channel, sender) =
            relm::Channel::new(
                move |new_channel: Result<Channel, Error>| match new_channel {
                    Ok(channel) => {
                        app_stream.emit(AppMsg::AddSubscription(channel));
                    }
                    Err(e) => {
                        app_stream.emit(AppMsg::Error(e));
                    }
                },
            );

        thread::spawn(move || {
            sender
                .send(Channel::from_id_or_name(&channel_name))
                .expect("Could not send channel");
        });
    }

    view! {
        gtk::Box {
            orientation: Vertical,

            gtk::Box {
                visible: self.model.add_subscription_visible,
                #[name="channel_name_entry"]
                gtk::Entry {
                    placeholder_text: Some("Channel Name or ID")
                },
                gtk::Button {
                    clicked => SubscriptionsPageMsg::AddSubscription,
                    image: Some(&gtk::Image::from_icon_name(Some("go-next-symbolic"), gtk::IconSize::LargeToolbar)),
                }
            },

            #[name="subscription_list"]
            LazyList<SubscriptionItem>
        }
    }
}
