use crate::errors::Error;
use crate::youtube_feed::feed::{Author, Feed};

use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;

use file_minidb::column::Column;
use file_minidb::serializer::Serializable;
use file_minidb::table::Table;
use file_minidb::types::ColumnType;
use file_minidb::values::Value;
use rayon::prelude::*;

const URL: &str = "https://www.youtube.com/feeds/videos.xml?channel_id=";

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct Channel {
    id: String,
    pub name: Option<String>,
}

impl Channel {
    pub fn new(id: &str) -> Self {
        Channel {
            id: id.to_string(),
            name: None,
        }
    }

    pub fn new_with_name(id: &str, name: &str) -> Self {
        Channel {
            id: id.to_string(),
            name: Some(name.to_string()),
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_name(&self) -> Option<String> {
        self.name.clone()
    }

    #[tokio::main]
    pub async fn get_feed(&self) -> Result<Feed, Error> {
        let url = URL.to_string() + &self.id;

        let content: Result<String, Error> = async {
            let res1 = reqwest::get(&url).await;

            if res1.is_err() {
                return Err(Error::networking());
            }

            let res2 = res1.unwrap().text().await;

            if res2.is_err() {
                return Err(Error::parsing(&self.id));
            }

            let res3 = res2.unwrap().replace("media:", "media_");

            Ok(res3)
        }
        .await;

        if let Err(e) = content {
            return Err(e);
        }

        let feed_res: Result<Feed, quick_xml::DeError> = quick_xml::de::from_str(&content.unwrap());

        if feed_res.is_err() {
            println!("Error parsing: {:?}", feed_res);
            return Err(Error::parsing(&self.id));
        }

        Ok(feed_res.unwrap())
    }
}

impl From<Author> for Channel {
    fn from(author: Author) -> Self {
        let id = author.uri.split("/").last().unwrap();
        let name = author.name;

        Channel::new_with_name(id, &name)
    }
}

#[derive(Clone, Debug)]
pub struct ChannelGroup {
    pub channels: Vec<Channel>,
}

impl ChannelGroup {
    pub fn new() -> Self {
        ChannelGroup {
            channels: Vec::new(),
        }
    }

    pub fn get_channels(&self) -> Vec<Channel> {
        self.channels.clone()
    }

    pub fn get_from_file(path: PathBuf) -> Result<ChannelGroup, Error> {
        let mut group = ChannelGroup::new();

        let mut subscriptions_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.clone())
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
            let table_res = Table::deserialize(contents);

            if let Err(_e) = table_res {
                return Err(Error::parsing(
                    &("Parsing subscriptions file ".to_string() + &path.to_string_lossy()),
                ));
            }

            let table = table_res.unwrap();

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

    pub fn add(&mut self, channel: Channel) {
        if !self.channels.contains(&channel) {
            self.channels.push(channel);
        }
    }

    pub async fn get_feed(&self) -> Result<Feed, Error> {
        let feeds: Vec<Result<Feed, _>> = self.channels.par_iter().map(|c| c.get_feed()).collect();

        if let Some(Err(e)) = feeds.clone().par_iter().find_any(|x| x.clone().is_err()) {
            return Err(e.clone());
        }

        Ok(Feed::combine(
            feeds
                .par_iter()
                .map(|f| f.as_ref().unwrap().clone())
                .collect(),
        ))
    }
}
