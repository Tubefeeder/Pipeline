use crate::gui::subscription_item::SubscriptionItem;
use crate::subscriptions::channel::ChannelGroup;

use gtk::prelude::*;
use gtk::SelectionMode;

use relm::ContainerWidget;
use relm::Relm;
use relm::Widget;

use relm_derive::{widget, Msg};

const FEED_PARTITION_SIZE: usize = 20;

#[derive(Msg)]
pub enum SubscriptionListMsg {
    SetSubscriptions(ChannelGroup),
    LoadMore,
}

pub struct SubscriptionListModel {
    subscriptions: ChannelGroup,
    loaded_elements: usize,
    relm: Relm<SubscriptionList>,
}

#[widget]
impl Widget for SubscriptionList {
    fn model(relm: &Relm<Self>, _: ()) -> SubscriptionListModel {
        SubscriptionListModel {
            subscriptions: ChannelGroup::new(),
            loaded_elements: 0,
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: SubscriptionListMsg) {
        match event {
            SubscriptionListMsg::SetSubscriptions(channels) => {
                self.model.subscriptions = channels;
                self.model.loaded_elements = 0;

                let subscription_list_clone = self.widgets.subscription_list.clone();
                self.widgets
                    .subscription_list
                    .forall(|w| subscription_list_clone.remove(w));

                self.model.relm.stream().emit(SubscriptionListMsg::LoadMore);
            }
            SubscriptionListMsg::LoadMore => {
                let loaded = self.model.loaded_elements;
                let channels = self.model.subscriptions.get_channels();
                if loaded < channels.len() {
                    let new_entries = &channels[self.model.loaded_elements
                        ..std::cmp::min(
                            self.model.loaded_elements + FEED_PARTITION_SIZE,
                            channels.len(),
                        )];

                    for entry in new_entries {
                        let _widget = self
                            .widgets
                            .subscription_list
                            .add_widget::<SubscriptionItem>(entry.clone());
                    }

                    self.model.loaded_elements += FEED_PARTITION_SIZE;
                }
            }
        }
    }

    view! {
        gtk::ScrolledWindow {
            hexpand: true,
            vexpand: true,
            edge_reached(_,_) => SubscriptionListMsg::LoadMore,
            gtk::Viewport {
                #[name="subscription_list"]
                gtk::ListBox {
                    selection_mode: SelectionMode::None,
                }
            }
        }
    }
}
