use crate::errors::Error;
use crate::subscriptions::Channel;
use crate::youtube_feed::Feed;

use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::{File, OpenOptions};
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

    /// Add a channel to the channel group.
    /// The channel group does not allow duplicates, these will be ignored.
    pub fn add(&mut self, channel: Channel) {
        if !self.channels.contains(&channel) {
            self.channels.push(channel);
            self.channels.sort();
        }
    }

    /// Removes a channel of the channel group.
    /// Only the id matters, the name is ignored in the comparison between the channels.
    pub fn remove(&mut self, channel: Channel) {
        self.channels = self
            .channels
            .clone()
            .into_iter()
            .filter(|c| c.clone() != channel)
            .collect();
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

    /// Parses the channel group from the file at the given path.
    /// The file must not exist, but it is created and a empty channel group will be returned.
    /// An error will be returned if the file could not be parsed.
    pub fn get_from_path(path: &PathBuf) -> Result<Self, Error> {
        let subscriptions_file_res = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.clone());

        if let Ok(mut subscriptions_file) = subscriptions_file_res {
            return ChannelGroup::get_from_file(path, &mut subscriptions_file);
        } else {
            return Err(Error::general_subscriptions(
                "opening",
                &path.to_string_lossy(),
            ));
        }
    }

    /// Parses the channel group from the given file.
    /// The file must not exist, but it is created and a empty channel group will be returned.
    /// An error will be returned if the file could not be parsed.
    fn get_from_file(path: &PathBuf, subscriptions_file: &mut File) -> Result<Self, Error> {
        let mut group = ChannelGroup::new();

        let mut contents = String::new();
        if subscriptions_file.read_to_string(&mut contents).is_ok() {
            if contents.is_empty() {
                let column_id = Column::key("channel_id", ColumnType::String);
                let table = Table::new(vec![column_id]).unwrap();
                let res = write!(subscriptions_file, "{}", table.serialize());

                if res.is_err() {
                    return Err(Error::general_subscriptions(
                        "writing",
                        &path.to_string_lossy(),
                    ));
                }
            } else {
                let table_res = Table::deserialize(contents);

                if let Err(_e) = table_res {
                    return Err(Error::parsing_subscriptions(&path.to_string_lossy()));
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
            return Ok(group);
        } else {
            return Err(Error::general_subscriptions(
                "reading",
                &path.to_string_lossy(),
            ));
        }
    }

    /// Writes the channel id's into the given file at the given path.
    /// The file must not exist, but it is created if it does not exist.
    pub fn write_to_path(&self, path: &PathBuf) -> Result<(), Error> {
        let subscriptions_file_res = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path.clone());

        if let Ok(mut subscriptions_file) = subscriptions_file_res {
            self.write_to_file(path, &mut subscriptions_file)
        } else {
            Err(Error::general_subscriptions(
                "opening",
                &path.to_string_lossy(),
            ))
        }
    }

    fn write_to_file(&self, path: &PathBuf, subscriptions_file: &mut File) -> Result<(), Error> {
        let column_id = Column::key("channel_id", ColumnType::String);
        let mut table = Table::new(vec![column_id]).unwrap();

        for channel in &self.channels {
            table
                .insert(vec![channel.get_id().into()])
                .expect("Could not append to table");
        }

        let write_res = write!(subscriptions_file, "{}", table.serialize());

        if write_res.is_err() {
            Err(Error::general_subscriptions(
                "writing",
                &path.to_string_lossy(),
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_group_is_empty() {
        let group = ChannelGroup::new();
        assert!(group.channels.is_empty());
    }

    #[test]
    fn test_add_new_channel() {
        let mut group = ChannelGroup::new();
        group.add(Channel::new("abcdef"));
        assert_eq!(group.channels.len(), 1);
        assert_eq!(group.channels, vec![Channel::new("abcdef")]);

        group.add(Channel::new("ghijkl"));
        assert_eq!(group.channels.len(), 2);
        assert_eq!(
            group.channels,
            vec![Channel::new("abcdef"), Channel::new("ghijkl")]
        );

        group.add(Channel::new("mnopqr"));
        assert_eq!(group.channels.len(), 3);
        assert_eq!(
            group.channels,
            vec![
                Channel::new("abcdef"),
                Channel::new("ghijkl"),
                Channel::new("mnopqr")
            ]
        );
    }

    #[test]
    fn test_channels_sorted() {
        let mut group = ChannelGroup::new();
        group.add(Channel::new_with_name("ghijkl", "a"));
        group.add(Channel::new_with_name("abcdef", "b"));
        group.add(Channel::new_with_name("mnopqr", "c"));
        assert_eq!(
            group.channels,
            vec![
                Channel::new_with_name("ghijkl", "a"),
                Channel::new_with_name("abcdef", "b"),
                Channel::new_with_name("mnopqr", "c")
            ]
        );
    }

    #[test]
    fn test_channels_sorted_mixed_case() {
        let mut group = ChannelGroup::new();
        group.add(Channel::new_with_name("ghijkl", "a"));
        group.add(Channel::new_with_name("abcdef", "B"));
        group.add(Channel::new_with_name("mnopqr", "c"));
        assert_eq!(
            group.channels,
            vec![
                Channel::new_with_name("ghijkl", "a"),
                Channel::new_with_name("abcdef", "B"),
                Channel::new_with_name("mnopqr", "c")
            ]
        );
    }

    #[test]
    fn test_insert_same_id() {
        let mut group = ChannelGroup::new();
        group.add(Channel::new_with_name("abcdef", "a"));
        group.add(Channel::new_with_name("abcdef", "b"));
        assert_eq!(group.channels.len(), 1);
        assert_eq!(group.channels, vec![Channel::new_with_name("abcdef", "a"),]);
    }

    #[test]
    fn test_remove() {
        let mut group = ChannelGroup::new();
        group.add(Channel::new("abcdef"));
        assert_eq!(group.channels.len(), 1);
        group.remove(Channel::new("abcdef"));
        assert_eq!(group.channels.len(), 0);
    }

    #[test]
    fn test_remove_with_name() {
        let mut group = ChannelGroup::new();
        group.add(Channel::new_with_name("abcdef", "a"));
        assert_eq!(group.channels.len(), 1);
        group.remove(Channel::new("abcdef"));
        assert_eq!(group.channels.len(), 0);
    }

    #[test]
    fn test_resolve_name() {
        let mut group1 = ChannelGroup::new();
        group1.add(Channel::new_with_name("abcdef", "a"));
        group1.add(Channel::new("ghijkl"));
        group1.add(Channel::new("mnopqr"));

        let mut group2 = ChannelGroup::new();
        group2.add(Channel::new_with_name("abcdef", "a2"));
        group2.add(Channel::new_with_name("ghijkl", "b"));
        group2.add(Channel::new_with_name("mnopqr", "c"));
        group2.add(Channel::new_with_name("stuvwx", "d"));

        group1.resolve_name(&group2);

        assert_eq!(group1.channels.len(), 3);
        assert_eq!(
            group1.channels,
            vec![
                Channel::new_with_name("abcdef", "a"),
                Channel::new_with_name("ghijkl", "b"),
                Channel::new_with_name("mnopqr", "c"),
            ]
        );
    }
}
