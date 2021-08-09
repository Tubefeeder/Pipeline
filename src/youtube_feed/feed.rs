/*
 * Copyright 2021 Julian Schmidhuber <github@schmiddi.anonaddy.com>
 *
 * This file is part of Tubefeeder.
 *
 * Tubefeeder is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Tubefeeder is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Tubefeeder.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

extern crate serde;

use crate::errors::Error;
use crate::filter::EntryFilterGroup;
use crate::subscriptions::{Channel, ChannelGroup};

use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use file_minidb::{
    column::Column, serializer::Serializable, table::Table, types::ColumnType, values::Value,
};

use chrono::NaiveDateTime;
use serde::Deserialize;

/// The youtube feed.
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Feed {
    #[serde(rename = "entry")]
    pub entries: Vec<Entry>,
}

/// A single entry
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Entry {
    pub title: String,
    pub author: Author,
    pub link: Link,
    #[serde(with = "date_serializer")]
    pub published: NaiveDateTime,
    #[serde(rename = "media_group")]
    pub media: Media,
}

/// The author of the video.
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Author {
    pub name: String,
    pub uri: String,
}

/// The media information of the video. Only used for the thumbnail.
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Media {
    #[serde(rename = "media_thumbnail")]
    pub thumbnail: Thumbnail,
}

/// The thumbnail link of the video.
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Thumbnail {
    pub url: String,
}

/// The link to the video.
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Link {
    pub href: String,
}

/// Deserializing `NativeDateTime`
mod date_serializer {
    use chrono::NaiveDateTime;

    use serde::{de::Error, Deserialize, Deserializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<NaiveDateTime, D::Error> {
        let time: String = Deserialize::deserialize(deserializer)?;
        Ok(
            NaiveDateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M:%S+00:00")
                .map_err(D::Error::custom)?,
        )
    }
}

impl Feed {
    /// Create a new, empty feed.
    pub fn empty() -> Self {
        Feed { entries: vec![] }
    }

    /// Combine many feeds into one feed. Will also sort the result by date.
    pub fn combine(feeds: Vec<Feed>) -> Feed {
        let mut entries: Vec<Entry> = Vec::new();

        for mut feed in feeds {
            entries.append(&mut feed.entries);
        }

        entries.sort_by_key(|e| e.published);
        entries.reverse();

        Feed { entries }
    }

    /// Extract the channels of the given feed.
    pub fn extract_channels(&self) -> ChannelGroup {
        let channels: Vec<Channel> = self
            .entries
            .iter()
            .map(|e| e.author.clone().into())
            .collect();

        ChannelGroup { channels }
    }

    /// Filter out entries matching one filter in the `EntryFilterGroup`.
    pub fn filter(&mut self, filter: &EntryFilterGroup) {
        self.entries = self
            .entries
            .iter()
            .filter(|e| !filter.matches_any(e))
            .cloned()
            .collect()
    }

    /// Parses the feed from the file at the given path.
    /// The file must not exist, but it is created and a empty feed will be returned.
    /// An error will be returned if the file could not be parsed.
    pub fn get_from_path(path: &PathBuf) -> Result<Self, Error> {
        let feed_file_res = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.clone());

        if let Ok(mut feed_file) = feed_file_res {
            Feed::get_from_file(path, &mut feed_file)
        } else {
            Err(Error::general_feed("opening", &path.to_string_lossy()))
        }
    }

    /// Parses the feed from the given file.
    /// The file must not exist, but it is created and a empty feed will be returned.
    /// An error will be returned if the file could not be parsed.
    fn get_from_file(path: &PathBuf, feed_file: &mut File) -> Result<Self, Error> {
        let mut feed_entries: Vec<Entry> = vec![];

        let mut contents = String::new();
        if feed_file.read_to_string(&mut contents).is_ok() {
            if contents.is_empty() {
                let column_url = Column::key("url", ColumnType::String);
                let column_title = Column::key("title", ColumnType::String);
                let column_published = Column::key("published", ColumnType::String);
                let column_author_name = Column::key("author_name", ColumnType::String);
                let column_author_uri = Column::key("author_uri", ColumnType::String);
                let column_thumbnail = Column::key("thumbnail", ColumnType::String);
                let table = Table::new(vec![
                    column_url,
                    column_title,
                    column_published,
                    column_author_name,
                    column_author_uri,
                    column_thumbnail,
                ])
                .unwrap();
                let res = write!(feed_file, "{}", table.serialize());

                if res.is_err() {
                    return Err(Error::general_feed("writing", &path.to_string_lossy()));
                }
            } else {
                let table_res = Table::deserialize(contents);

                if let Err(_e) = table_res {
                    return Err(Error::parsing_feed(&path.to_string_lossy()));
                }

                let table = table_res.unwrap();

                let entries = table.get_entries();

                for entry in entries {
                    let values: Vec<Value> = entry.get_values();
                    let url: String = values[0].clone().try_into().unwrap();
                    let title: String = values[1].clone().try_into().unwrap();
                    let published: String = values[2].clone().try_into().unwrap();
                    let author_name: String = values[3].clone().try_into().unwrap();
                    let author_uri: String = values[4].clone().try_into().unwrap();
                    let thumbnail: String = values[5].clone().try_into().unwrap();

                    let date = NaiveDateTime::parse_from_str(&published, "%Y-%m-%dT%H:%M:%S+00:00");

                    if date.is_err() {
                        return Err(Error::parsing_feed(&path.to_string_lossy()));
                    }

                    let author = Author {
                        name: author_name,
                        uri: author_uri,
                    };

                    let media = Media {
                        thumbnail: Thumbnail { url: thumbnail },
                    };

                    let link = Link { href: url };

                    let feed_entry = Entry {
                        title,
                        author,
                        link,
                        published: date.unwrap(),
                        media,
                    };

                    feed_entries.push(feed_entry);
                }
            }
            Ok(Feed {
                entries: feed_entries,
            })
        } else {
            Err(Error::general_feed("reading", &path.to_string_lossy()))
        }
    }

    /// Writes the channel id's into the given file at the given path.
    /// The file must not exist, but it is created if it does not exist.
    pub fn write_to_path(&self, path: &PathBuf) -> Result<(), Error> {
        let feed_file_res = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path.clone());

        if let Ok(mut feed_file) = feed_file_res {
            self.write_to_file(path, &mut feed_file)
        } else {
            Err(Error::general_feed("opening", &path.to_string_lossy()))
        }
    }

    fn write_to_file(&self, path: &PathBuf, feed_file: &mut File) -> Result<(), Error> {
        let column_url = Column::key("url", ColumnType::String);
        let column_title = Column::key("title", ColumnType::String);
        let column_published = Column::key("published", ColumnType::String);
        let column_author_name = Column::key("author_name", ColumnType::String);
        let column_author_uri = Column::key("author_uri", ColumnType::String);
        let column_thumbnail = Column::key("thumbnail", ColumnType::String);
        let mut table = Table::new(vec![
            column_url,
            column_title,
            column_published,
            column_author_name,
            column_author_uri,
            column_thumbnail,
        ])
        .unwrap();

        for entry in &self.entries {
            table
                .insert(vec![
                    entry.clone().link.href.into(),
                    entry.clone().title.into(),
                    entry
                        .clone()
                        .published
                        .format("%Y-%m-%dT%H:%M:%S+00:00")
                        .to_string()
                        .into(),
                    entry.clone().author.name.into(),
                    entry.clone().author.uri.into(),
                    entry.clone().media.thumbnail.url.into(),
                ])
                .expect("Could not append to table");
        }

        let write_res = write!(feed_file, "{}", table.serialize());

        if write_res.is_err() {
            Err(Error::general_feed("writing", &path.to_string_lossy()))
        } else {
            Ok(())
        }
    }
}

impl Entry {
    /// Play the video using mpv.
    pub fn play(&self) -> Result<Child, std::io::Error> {
        Command::new("mpv")
            .arg(&self.link.href)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::NaiveDate;

    impl Author {
        fn new() -> Self {
            Author {
                name: "".to_string(),
                uri: "".to_string(),
            }
        }
    }

    impl Link {
        fn new() -> Self {
            Link {
                href: "".to_string(),
            }
        }
    }

    impl Thumbnail {
        fn new() -> Self {
            Thumbnail {
                url: "".to_string(),
            }
        }
    }

    impl Media {
        fn new() -> Self {
            Media {
                thumbnail: Thumbnail::new(),
            }
        }
    }

    impl Entry {
        fn new(title: &str, date: NaiveDateTime) -> Self {
            Entry {
                title: title.to_string(),
                author: Author::new(),
                link: Link::new(),
                published: date,
                media: Media::new(),
            }
        }
    }

    #[test]
    fn test_empty_feed() {
        let feed = Feed::empty();
        assert_eq!(feed.entries.len(), 0);
    }

    #[test]
    fn test_combine() {
        let mut feed1 = Feed::empty();
        feed1.entries.push(Entry::new(
            "mno",
            NaiveDate::from_ymd(2021, 1, 5).and_hms(0, 0, 0),
        ));
        feed1.entries.push(Entry::new(
            "jkl",
            NaiveDate::from_ymd(2021, 1, 4).and_hms(0, 0, 0),
        ));
        feed1.entries.push(Entry::new(
            "def",
            NaiveDate::from_ymd(2021, 1, 2).and_hms(0, 0, 0),
        ));

        let mut feed2 = Feed::empty();
        feed2.entries.push(Entry::new(
            "stu",
            NaiveDate::from_ymd(2021, 1, 7).and_hms(0, 0, 0),
        ));
        feed2.entries.push(Entry::new(
            "ghi",
            NaiveDate::from_ymd(2021, 1, 3).and_hms(0, 0, 0),
        ));
        feed2.entries.push(Entry::new(
            "abc",
            NaiveDate::from_ymd(2021, 1, 1).and_hms(0, 0, 0),
        ));

        let mut feed3 = Feed::empty();
        feed3.entries.push(Entry::new(
            "pqr",
            NaiveDate::from_ymd(2021, 1, 6).and_hms(0, 0, 0),
        ));
        feed3.entries.push(Entry::new(
            "vwx",
            NaiveDate::from_ymd(2021, 1, 8).and_hms(0, 0, 0),
        ));

        let combined = Feed::combine(vec![feed1, feed2, feed3]);

        assert_eq!(
            combined.entries,
            vec![
                Entry::new("vwx", NaiveDate::from_ymd(2021, 1, 8).and_hms(0, 0, 0)),
                Entry::new("stu", NaiveDate::from_ymd(2021, 1, 7).and_hms(0, 0, 0)),
                Entry::new("pqr", NaiveDate::from_ymd(2021, 1, 6).and_hms(0, 0, 0)),
                Entry::new("mno", NaiveDate::from_ymd(2021, 1, 5).and_hms(0, 0, 0)),
                Entry::new("jkl", NaiveDate::from_ymd(2021, 1, 4).and_hms(0, 0, 0)),
                Entry::new("ghi", NaiveDate::from_ymd(2021, 1, 3).and_hms(0, 0, 0)),
                Entry::new("def", NaiveDate::from_ymd(2021, 1, 2).and_hms(0, 0, 0)),
                Entry::new("abc", NaiveDate::from_ymd(2021, 1, 1).and_hms(0, 0, 0))
            ]
        )
    }
}
