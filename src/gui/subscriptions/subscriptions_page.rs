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

use crate::gui::subscriptions::subscription_item::SubscriptionItem;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use gtk::prelude::*;
use gtk::Orientation::Vertical;
use gtk::{Container, Label, ListBoxRow};
use relm::{Channel, ContainerWidget, Relm, Sender, Widget};
use relm_derive::{widget, Msg};
use tf_join::{AnySubscription, AnySubscriptionList, SubscriptionEvent};
use tf_observer::{Observable, Observer};
use tf_yt::YTSubscription;

#[derive(Msg)]
pub enum SubscriptionsPageMsg {
    ToggleAddSubscription,
    AddSubscription,
    NewSubscription(AnySubscription),
    RemoveSubscription(AnySubscription),
}

pub struct SubscriptionsPageModel {
    relm: Relm<SubscriptionsPage>,
    add_subscription_visible: bool,
    subscription_list: AnySubscriptionList,
    _subscription_observer: Arc<Mutex<Box<dyn Observer<SubscriptionEvent> + Send>>>,
    subscription_items: HashMap<AnySubscription, relm::Component<SubscriptionItem>>,
}

#[widget]
impl Widget for SubscriptionsPage {
    fn model(relm: &Relm<Self>, subscription_list: AnySubscriptionList) -> SubscriptionsPageModel {
        let relm_clone = relm.clone();
        let (_channel, sender) = Channel::new(move |msg| relm_clone.stream().emit(msg));

        let observer = Arc::new(Mutex::new(Box::new(SubscriptionsPageObserver { sender })
            as Box<dyn Observer<SubscriptionEvent> + Send>));

        let mut subscription_list_clone = subscription_list.clone();
        subscription_list_clone
            .iter()
            .for_each(|s| relm.stream().emit(SubscriptionsPageMsg::NewSubscription(s)));
        subscription_list_clone.attach(Arc::downgrade(&observer));

        SubscriptionsPageModel {
            relm: relm.clone(),
            add_subscription_visible: false,
            subscription_list: subscription_list_clone,
            _subscription_observer: observer,
            subscription_items: HashMap::new(),
        }
    }

    fn update(&mut self, event: SubscriptionsPageMsg) {
        match event {
            SubscriptionsPageMsg::ToggleAddSubscription => {
                self.model.add_subscription_visible = !self.model.add_subscription_visible;
            }
            SubscriptionsPageMsg::AddSubscription => self.add_subscription(),
            SubscriptionsPageMsg::NewSubscription(sub) => self.new_subscription(sub),
            SubscriptionsPageMsg::RemoveSubscription(sub) => self.remove_subscription(sub),
        }
    }

    fn add_subscription(&mut self) {
        let channel_id = self.widgets.channel_id_entry.text();

        self.widgets.channel_id_entry.set_text("");
        self.model
            .relm
            .stream()
            .emit(SubscriptionsPageMsg::ToggleAddSubscription);

        self.model
            .subscription_list
            .add(YTSubscription::new(&channel_id).into());
    }

    fn new_subscription(&mut self, sub: AnySubscription) {
        if self.model.subscription_items.get(&sub).is_none() {
            let sub_item = self
                .widgets
                .subscription_list
                .add_widget::<SubscriptionItem>((
                    sub.clone(),
                    self.model.subscription_list.clone(),
                ));

            self.model.subscription_items.insert(sub, sub_item);
        }
    }

    fn remove_subscription(&mut self, sub: AnySubscription) {
        if let Some(filter_item) = self.model.subscription_items.get(&sub) {
            self.widgets.subscription_list.remove(filter_item.widget());
            self.model.subscription_items.remove(&sub);
        }
    }

    fn init_view(&mut self) {
        self.widgets.channel_entry_box.hide();

        self.widgets.subscription_list.set_sort_func(Some(Box::new(
            |l1: &ListBoxRow, l2: &ListBoxRow| {
                // The gtk::Box inside the gtk::ListBoxRow.
                let b1 = l1.child().unwrap();
                let b2 = l2.child().unwrap();

                let c1 = b1.clone().dynamic_cast::<Container>().unwrap();
                let c2 = b2.clone().dynamic_cast::<Container>().unwrap();

                // The gtk::Box inside the gtk::box b1 and b2.
                let d1 = &c1.children()[1];
                let d2 = &c2.children()[1];

                let e1 = d1.clone().dynamic_cast::<Container>().unwrap();
                let e2 = d2.clone().dynamic_cast::<Container>().unwrap();

                // The gtk::Label inside the gtk::box b1 and b2.
                let f1 = &e1.children()[0];
                let f2 = &e2.children()[0];

                let g1 = f1.clone().dynamic_cast::<Label>().unwrap();
                let g2 = f2.clone().dynamic_cast::<Label>().unwrap();

                let s1 = g1.text().as_str().to_string();
                let s2 = g2.text().as_str().to_string();

                if s1 < s2 {
                    -1
                } else {
                    1
                }
            },
        )));
    }

    view! {
        gtk::Box {
            orientation: Vertical,

            #[name="channel_entry_box"]
            gtk::Box {
                visible: self.model.add_subscription_visible,
                #[name="channel_id_entry"]
                gtk::Entry {
                    placeholder_text: Some("Channel ID")
                },
                gtk::Button {
                    clicked => SubscriptionsPageMsg::AddSubscription,
                    image: Some(&gtk::Image::from_icon_name(Some("go-next-symbolic"), gtk::IconSize::LargeToolbar)),
                }
            },

            gtk::ScrolledWindow {
                hexpand: true,
                vexpand: true,
                gtk::Viewport {
                    #[name="subscription_list"]
                    gtk::ListBox {
                        selection_mode: gtk::SelectionMode::None

                    }
                }
            }
        }
    }
}

pub struct SubscriptionsPageObserver {
    sender: Sender<SubscriptionsPageMsg>,
}

impl Observer<SubscriptionEvent> for SubscriptionsPageObserver {
    fn notify(&mut self, message: SubscriptionEvent) {
        match message {
            SubscriptionEvent::Add(sub) => {
                let _ = self.sender.send(SubscriptionsPageMsg::NewSubscription(sub));
            }
            SubscriptionEvent::Remove(sub) => {
                let _ = self
                    .sender
                    .send(SubscriptionsPageMsg::RemoveSubscription(sub));
            }
        }
    }
}
