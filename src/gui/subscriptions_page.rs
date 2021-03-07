use crate::gui::lazy_list::{LazyList, LazyListMsg, ListElementBuilder};
use crate::gui::subscription_item::SubscriptionItem;
use crate::subscriptions::channel::{Channel, ChannelGroup};

use gtk::prelude::*;
use gtk::Orientation::Vertical;
use relm::Widget;
use relm_derive::{widget, Msg};

pub struct SubscriptionElementBuilder {
    chunks: Vec<Vec<Channel>>,
}

impl SubscriptionElementBuilder {
    fn new(group: ChannelGroup) -> Self {
        SubscriptionElementBuilder {
            chunks: group
                .channels
                .chunks(20)
                .map(|slice| slice.to_vec())
                .collect::<Vec<Vec<Channel>>>(),
        }
    }
}

impl ListElementBuilder<SubscriptionItem> for SubscriptionElementBuilder {
    fn poll(&mut self) -> Vec<Channel> {
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
}

#[widget]
impl Widget for SubscriptionsPage {
    fn model() -> () {}

    fn update(&mut self, event: SubscriptionsPageMsg) {
        match event {
            SubscriptionsPageMsg::SetSubscriptions(channels) => {
                self.components
                    .subscription_list
                    .emit(LazyListMsg::SetListElementBuilder(Box::new(
                        SubscriptionElementBuilder::new(channels),
                    )));
            }
        }
    }

    view! {
        gtk::Box {
            orientation: Vertical,
            #[name="subscription_list"]
            LazyList<SubscriptionItem>
        }
    }
}
