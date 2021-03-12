extern crate serde;

use crate::subscriptions::{Channel, ChannelGroup};

use std::process::{Child, Command, Stdio};

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

        return ChannelGroup { channels };
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
