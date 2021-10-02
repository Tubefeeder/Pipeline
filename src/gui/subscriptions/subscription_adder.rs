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

use std::str::FromStr;

use gtk::prelude::*;
use relm::{Relm, Widget};
use relm_derive::{widget, Msg};
use tf_join::{AnySubscription, AnySubscriptionList, Platform};
use tf_pt::PTSubscription;
use tf_yt::YTSubscription;

#[derive(Msg)]
pub enum SubscriptionAdderMsg {
    ToggleVisible,
    AddSubscription,
    ChangePlatform(Platform),
}

pub struct SubscriptionAdderModel {
    relm: Relm<SubscriptionAdder>,
    visible: bool,
    subscription_list: AnySubscriptionList,
    platform: Platform,
}

#[widget]
impl Widget for SubscriptionAdder {
    fn model(relm: &Relm<Self>, subscription_list: AnySubscriptionList) -> SubscriptionAdderModel {
        SubscriptionAdderModel {
            relm: relm.clone(),
            visible: false,
            subscription_list,
            platform: Platform::Youtube,
        }
    }

    fn update(&mut self, event: SubscriptionAdderMsg) {
        match event {
            SubscriptionAdderMsg::ToggleVisible => {
                self.model.visible = !self.model.visible;
            }
            SubscriptionAdderMsg::AddSubscription => self.add_subscription(),
            SubscriptionAdderMsg::ChangePlatform(platform) => {
                self.model.platform = platform;
            }
        }
    }

    fn add_subscription(&mut self) {
        let channel_id_or_name = self.widgets.channel_id_or_name_entry.text();
        let base_url = self.widgets.base_url_entry.text();

        self.widgets.channel_id_or_name_entry.set_text("");
        self.widgets.base_url_entry.set_text("");

        self.model
            .relm
            .stream()
            .emit(SubscriptionAdderMsg::ToggleVisible);

        let sub_list = self.model.subscription_list.clone();
        let platform = self.model.platform.clone();

        std::thread::spawn(move || {
            // TODO: Differentiate between platforms.
            let sub_res: Result<AnySubscription, tf_core::Error> =
                tokio::runtime::Runtime::new().unwrap().block_on(async {
                    match platform {
                        Platform::Youtube => YTSubscription::from_id_or_name(&channel_id_or_name)
                            .await
                            .map(|s| s.into()),
                        Platform::Peertube => {
                            Ok(PTSubscription::new(&base_url, &channel_id_or_name).into())
                        }
                    }
                });

            // TODO: Error handling
            if let Ok(sub) = sub_res {
                sub_list.add(sub);
            }
        });
    }

    fn init_view(&mut self) {
        let platform_combobox = gtk::ComboBoxText::new();

        for value in Platform::values() {
            let item = value.to_string();
            platform_combobox.append(Some(&item), &item);
        }
        platform_combobox.set_active(Some(0));

        let platform_combobox_clone = platform_combobox.clone();

        relm::connect!(
            self.model.relm,
            platform_combobox.clone(),
            connect_changed(_),
            SubscriptionAdderMsg::ChangePlatform(Platform::from(
                Platform::from_str(
                    &platform_combobox_clone
                        .active_id()
                        .unwrap_or(glib::GString::from("")),
                )
                .unwrap_or(Platform::Youtube),
            ))
        );

        self.widgets
            .platform_combobox_holder
            .add(&platform_combobox);
        self.widgets.platform_combobox_holder.show_all();
    }

    view! {
        #[name="channel_entry_box"]
        gtk::Box {
            visible: self.model.visible,
            #[name="platform_combobox_holder"]
            gtk::Box {

            },

            #[name="base_url_entry"]
            gtk::Entry {
                placeholder_text: Some("Base URL"),
                visible: self.model.platform == Platform::Peertube
            },
            #[name="channel_id_or_name_entry"]
            gtk::Entry {
                placeholder_text: Some("Channel ID or Name")
            },
            gtk::Button {
                clicked => SubscriptionAdderMsg::AddSubscription,
                image: Some(&gtk::Image::from_icon_name(Some("go-next-symbolic"), gtk::IconSize::LargeToolbar)),
            }
        },
    }
}
