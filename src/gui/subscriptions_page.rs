use crate::gui::subscription_list::{SubscriptionList, SubscriptionListMsg};
use crate::subscriptions::channel::ChannelGroup;

use gtk::prelude::*;
use gtk::Orientation::Vertical;
use relm::Widget;
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum SubscriptionsPageMsg {
    SetSubscriptions(ChannelGroup),
}

#[widget]
impl Widget for SubscriptionsPage {
    fn model() -> () {}

    fn update(&mut self, event: SubscriptionsPageMsg) {
        match event {
            SubscriptionsPageMsg::SetSubscriptions(channels) => {
                self.components
                    .subscription_list
                    .emit(SubscriptionListMsg::SetSubscriptions(channels));
            }
        }
    }

    view! {
        gtk::Box {
            orientation: Vertical,
            #[name="subscription_list"]
            SubscriptionList
        }
    }
}
