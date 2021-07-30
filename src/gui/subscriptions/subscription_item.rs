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

use crate::gui::app::AppMsg;
use crate::gui::{get_font_size, FONT_RATIO};
use crate::subscriptions::Channel;

use gtk::prelude::*;
use gtk::Align;
use gtk::Orientation::Vertical;
use pango::{AttrList, Attribute, EllipsizeMode};
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum SubscriptionItemMsg {
    Remove,
}

pub struct SubscriptionsItemModel {
    channel: Channel,
    app_stream: StreamHandle<AppMsg>,
}

#[widget]
impl Widget for SubscriptionItem {
    fn model(
        _: &Relm<Self>,
        (channel, app_stream): (Channel, StreamHandle<AppMsg>),
    ) -> SubscriptionsItemModel {
        SubscriptionsItemModel {
            channel,
            app_stream,
        }
    }

    fn update(&mut self, event: SubscriptionItemMsg) {
        match event {
            SubscriptionItemMsg::Remove => {
                self.model
                    .app_stream
                    .emit(AppMsg::RemoveSubscription(self.model.channel.clone()));
            }
        }
    }

    fn init_view(&mut self) {
        let font_size = get_font_size();
        let name_attr_list = AttrList::new();
        name_attr_list.insert(Attribute::new_size(font_size * pango::SCALE).unwrap());
        self.widgets
            .label_name
            .set_attributes(Some(&name_attr_list));

        let id_attr_list = AttrList::new();
        id_attr_list.insert(
            Attribute::new_size((FONT_RATIO * (font_size * pango::SCALE) as f32) as i32).unwrap(),
        );
        self.widgets.label_id.set_attributes(Some(&id_attr_list));
    }

    view! {
        gtk::ListBoxRow {
            gtk::Box {
                gtk::Button {
                    image: Some(&gtk::Image::from_icon_name(Some("list-remove-symbolic"), gtk::IconSize::LargeToolbar)),
                    clicked => SubscriptionItemMsg::Remove,
                },
                gtk::Box {
                    orientation: Vertical,
                    #[name="label_name"]
                    gtk::Label {
                        text: &self.model.channel.get_name().unwrap_or("".to_string()),
                        ellipsize: EllipsizeMode::End,
                        halign: Align::Start
                    },
                    #[name="label_id"]
                    gtk::Label {
                        text: &self.model.channel.get_id(),
                        ellipsize: EllipsizeMode::End,
                        halign: Align::Start
                    },
                }
            }
        }
    }
}
