use crate::gui::app::AppMsg;
use crate::gui::lazy_list::{LazyList, LazyListMsg, ListElementBuilder};
use crate::gui::subscription_item::SubscriptionItem;
use crate::subscriptions::{Channel, ChannelGroup};

use gtk::prelude::*;
use gtk::Orientation::Vertical;
use relm::{Relm, StreamHandle, Widget};
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
                        SubscriptionElementBuilder::new(channels),
                    )));
            }
            SubscriptionsPageMsg::ToggleAddSubscription => {
                self.model.add_subscription_visible = !self.model.add_subscription_visible;
            }
            SubscriptionsPageMsg::AddSubscription => {
                let channel_id = &self.widgets.channel_id_entry.get_text();

                self.widgets.channel_id_entry.set_text("");
                self.model
                    .relm
                    .stream()
                    .emit(SubscriptionsPageMsg::ToggleAddSubscription);

                let new_channel = Channel::new(channel_id);
                self.model
                    .app_stream
                    .emit(AppMsg::AddSubscription(new_channel));
            }
        }
    }

    view! {
        gtk::Box {
            orientation: Vertical,

            gtk::Box {
                visible: self.model.add_subscription_visible,
                #[name="channel_id_entry"]
                gtk::Entry {
                    placeholder_text: Some("Channel ID")
                },
                gtk::Button {
                    clicked => SubscriptionsPageMsg::AddSubscription,
                    image: Some(&gtk::Image::from_icon_name(Some("go-next"), gtk::IconSize::LargeToolbar)),
                }
            },

            #[name="subscription_list"]
            LazyList<SubscriptionItem>
        }
    }
}
