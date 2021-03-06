use crate::gui::feed_item::{FeedListItem, FeedListItemMsg};
use crate::youtube_feed::feed::Feed;

use gtk::prelude::*;
use gtk::ListBoxRow;
use gtk::SelectionMode;
use relm::Component;
use relm::ContainerWidget;
use relm::Relm;
use relm::Widget;
use relm_derive::{widget, Msg};

const FEED_PARTITION_SIZE: usize = 10;

#[derive(Msg)]
pub enum FeedListMsg {
    RowActivated(ListBoxRow),
    SetFeed(Feed),
    LoadMore,
}

pub struct FeedListModel {
    feed: Feed,
    elements: Vec<Component<FeedListItem>>,
    relm: Relm<FeedList>,
    loaded_elements: usize,
}

#[widget]
impl Widget for FeedList {
    fn model(relm: &Relm<Self>, _: ()) -> FeedListModel {
        FeedListModel {
            feed: Feed::empty(),
            elements: vec![],
            relm: relm.clone(),
            loaded_elements: 0,
        }
    }

    fn update(&mut self, event: FeedListMsg) {
        match event {
            FeedListMsg::RowActivated(row) => {
                let index = self
                    .widgets
                    .feed_list
                    .get_children()
                    .iter()
                    .position(|x| x.clone() == row)
                    .unwrap();

                let entry = &self.model.feed.entries[index];

                entry.play();
            }
            FeedListMsg::SetFeed(feed) => {
                self.model.elements.clear();
                self.model.feed = feed;
                self.model.loaded_elements = 0;

                let feed_list_clone = self.widgets.feed_list.clone();
                self.widgets.feed_list.forall(|w| feed_list_clone.remove(w));

                self.model.relm.stream().emit(FeedListMsg::LoadMore);
            }
            FeedListMsg::LoadMore => {
                let loaded = self.model.loaded_elements;
                let entries = self.model.feed.entries.clone();

                if loaded < entries.len() {
                    let new_entries = &entries[self.model.loaded_elements
                        ..std::cmp::min(
                            self.model.loaded_elements + FEED_PARTITION_SIZE,
                            entries.len(),
                        )];

                    for entry in new_entries {
                        let widget = self
                            .widgets
                            .feed_list
                            .add_widget::<FeedListItem>(entry.clone());
                        widget.emit(FeedListItemMsg::SetImage);
                        self.model.elements.push(widget);
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
            edge_reached(_,_) => FeedListMsg::LoadMore,
            gtk::Viewport {
                #[name="feed_list"]
                gtk::ListBox {
                    selection_mode: SelectionMode::None,
                    row_activated(_, row) => FeedListMsg::RowActivated(row.clone())
                }
            }
        }
    }
}
