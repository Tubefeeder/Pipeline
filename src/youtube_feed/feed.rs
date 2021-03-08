extern crate serde;

use crate::subscriptions::{Channel, ChannelGroup};

use std::process::{Command, Stdio};

use chrono::NaiveDateTime;
use serde::Deserialize;

/// The youtube feed.
#[derive(Debug, Deserialize, Clone)]
pub struct Feed {
    #[serde(rename = "entry")]
    pub entries: Vec<Entry>,
}

/// A single entry
#[derive(Debug, Deserialize, Clone)]
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
#[derive(Debug, Deserialize, Clone)]
pub struct Author {
    pub name: String,
    pub uri: String,
}

/// The media information of the video. Only used for the thumbnail.
#[derive(Debug, Deserialize, Clone)]
pub struct Media {
    #[serde(rename = "media_thumbnail")]
    pub thumbnail: Thumbnail,
}

/// The thumbnail link of the video.
#[derive(Debug, Deserialize, Clone)]
pub struct Thumbnail {
    pub url: String,
}

/// The link to the video.
#[derive(Debug, Deserialize, Clone)]
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

    /// Combine two feeds into one feed. Will also sort the result by date.
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
    pub fn play(&self) {
        let _res = Command::new("mpv")
            .arg(&self.link.href)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
}
