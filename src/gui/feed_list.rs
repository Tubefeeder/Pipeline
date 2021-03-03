use crate::errors::Error;
use crate::gui::feed_item::FeedListItem;
use crate::subscriptions::channel::Channel;
use crate::subscriptions::channel::ChannelGroup;
use crate::youtube_feed::feed::Feed;

use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::{Read, Write};

use file_minidb::column::Column;
use file_minidb::serializer::Serializable;
use file_minidb::table::Table;
use file_minidb::types::ColumnType;
use file_minidb::values::Value;

use relm::Component;

use gtk::prelude::*;
use gtk::ListBoxRow;
use gtk::SelectionMode;

use relm::ContainerWidget;
use relm::Relm;
use relm::Widget;

use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum FeedListMsg {
    Reload,
    RowActivated(ListBoxRow),
}

pub struct FeedListModel {
    feed: Feed,
    subscriptions: ChannelGroup,
    elements: Vec<Component<FeedListItem>>,
}

impl FeedList {
    fn get_subscriptions() -> Result<ChannelGroup, Error> {
        let mut group = ChannelGroup::new();

        let mut user_data_dir =
            glib::get_user_data_dir().expect("could not get user data directory");
        user_data_dir.push("tubefeeder");

        if !user_data_dir.exists() {
            std::fs::create_dir_all(user_data_dir.clone()).expect("could not create user data dir");
        }

        let mut subscriptions_file_path = user_data_dir.clone();
        subscriptions_file_path.push("subscriptions.db");

        let mut subscriptions_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(subscriptions_file_path)
            .expect("could not open subscriptions file");

        let mut contents = String::new();
        subscriptions_file
            .read_to_string(&mut contents)
            .expect("could not read subscriptions file");

        if contents.is_empty() {
            let column_id = Column::key("channel_id", ColumnType::String);
            let table = Table::new(vec![column_id]).unwrap();
            write!(subscriptions_file, "{}", table.serialize())
                .expect("could not write to subscriptions file");
        } else {
            let table =
                Table::deserialize(contents).expect("could not deserialize subscriptions file");
            let entries = table.get_entries();

            for entry in entries {
                let values: Vec<Value> = entry.get_values();
                let channel_id: Value = values[0].clone();
                let channel_id_str: String = channel_id.try_into().unwrap();
                group.add(Channel::new(&channel_id_str));
            }
        }
        Ok(group)
    }
}

#[widget]
impl Widget for FeedList {
    fn model(_: &Relm<Self>, _: ()) -> FeedListModel {
        FeedListModel {
            feed: Feed::empty(),
            subscriptions: FeedList::get_subscriptions().unwrap(),
            elements: vec![],
        }
    }

    fn update(&mut self, event: FeedListMsg) {
        match event {
            FeedListMsg::Reload => {
                let feed = futures::executor::block_on(self.model.subscriptions.get_feed());

                if let Err(_e) = feed {
                    return;
                }

                self.model.feed = feed.unwrap();

                for entry in &self.model.feed.entries {
                    let widget = self
                        .widgets
                        .feed_list
                        .add_widget::<FeedListItem>(entry.clone());
                    self.model.elements.push(widget);
                }
            }
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
        }
    }

    view! {
        gtk::ScrolledWindow {
            hexpand: true,
            vexpand: true,
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
