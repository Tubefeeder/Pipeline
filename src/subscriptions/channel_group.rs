use crate::errors::Error;
use crate::subscriptions::Channel;
use crate::youtube_feed::Feed;

use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;

use file_minidb::{
    column::Column, serializer::Serializable, table::Table, types::ColumnType, values::Value,
};
use rayon::prelude::*;

/// A struct for holding a group of channels without duplicates.
#[derive(Clone, Debug)]
pub struct ChannelGroup {
    pub channels: Vec<Channel>,
}

impl ChannelGroup {
    /// Create a new, empty channel group.
    pub fn new() -> Self {
        ChannelGroup {
            channels: Vec::new(),
        }
    }

    /// Parses the channel group from the file at the given path.
    /// The file must not exist, but it is created afterwards and a empty channel group will be returned.
    /// An error will be returned if the file could not be parsed.
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

    /// Add a channel to the channel group.
    /// The channel group does not allow duplicates, these will be ignored.
    pub fn add(&mut self, channel: Channel) {
        if !self.channels.contains(&channel) {
            self.channels.push(channel);
            self.channels.sort();
        }
    }

    /// Get the feed of the entire channel group.
    /// Results in an error when one channel of the channel group returns an error for the feed.
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

    /// Resolves the name of this channel group using another channel group.
    /// Will look up the name of each channel in `self` in the `other` channel group and set the name
    /// in `self` if not already set.
    pub fn resolve_name(&mut self, other: &ChannelGroup) {
        let hashmap: HashMap<String, Option<String>> = other
            .channels
            .iter()
            .map(|c| (c.get_id(), c.get_name()))
            .collect();

        self.channels = self
            .channels
            .iter()
            .map(|c| {
                let mut result = c.clone();
                if c.name == None {
                    if let Some(name) = hashmap.get(&c.get_id()) {
                        result.name = name.clone()
                    }
                }
                result
            })
            .collect();

        self.channels.sort();
    }
}
