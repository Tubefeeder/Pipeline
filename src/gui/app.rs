use crate::gui::feed_list::FeedList;
use crate::subscriptions::channel::{Channel, ChannelGroup};
use crate::youtube_feed::feed::Feed;

use std::convert::TryInto;
use std::io::Read;
use std::{fs::OpenOptions, io::Write};

use file_minidb::serializer::Serializable;
use file_minidb::table::Table;
use file_minidb::types::ColumnType;
use file_minidb::{column::Column, values::Value};

use libhandy::ApplicationWindow;

use gtk::prelude::*;
use gtk::{Application, Builder, Button, Label, ListBox};

#[derive(Clone)]
pub struct App {
    btn_reload: Button,
    network_error_label: Label,

    feed_list: FeedList,
    subscriptions: ChannelGroup,
}

impl App {
    pub fn init(application: &Application) {
        let mut app = App::new(application);
        app.setup_reload();
        app.reload();
    }

    fn new(application: &Application) -> Self {
        let feed_src = include_str!("../../glade/feed.glade");
        let builder = Builder::from_string(feed_src);

        let window: ApplicationWindow = builder.get_object("window").expect("could not get window");
        window.set_application(Some(application));

        let feed_list: ListBox = builder.get_object("feed_list").expect("could not get feed");

        let list: FeedList = FeedList::new(feed_list);

        window.show_all();

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

        App {
            btn_reload: builder
                .get_object("feed_button_reload")
                .expect("could not get reload button"),
            network_error_label: builder
                .get_object("label_error_network")
                .expect("could not get network error label"),

            feed_list: list,
            subscriptions: group,
        }
    }

    fn setup_reload(&self) {
        let clone = self.clone();

        self.btn_reload
            .connect_clicked(move |_| clone.clone().reload());
    }

    fn reload(&mut self) {
        let feed = futures::executor::block_on(self.subscriptions.get_feed());

        if let Err(_e) = feed {
            self.network_error_label.set_visible(true);
            self.feed_list.set_feed(Feed::empty());
        } else {
            self.network_error_label.set_visible(false);
            self.feed_list.set_feed(feed.unwrap());
        }
    }
}
