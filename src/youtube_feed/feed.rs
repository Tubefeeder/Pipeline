extern crate serde;

use std::process::Command;
use std::process::Stdio;

use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Feed {
    #[serde(rename = "entry")]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Author {
    pub name: String,
    pub uri: String,
}

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

#[derive(Debug, Deserialize, Clone)]
pub struct Media {
    #[serde(rename = "media_thumbnail")]
    pub thumbnail: Thumbnail,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Thumbnail {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Link {
    pub href: String,
}

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
    pub fn empty() -> Self {
        Feed { entries: vec![] }
    }

    pub fn combine(feeds: Vec<Feed>) -> Feed {
        let mut entries: Vec<Entry> = Vec::new();

        for mut feed in feeds {
            entries.append(&mut feed.entries);
        }

        entries.sort_by_key(|e| e.published);
        entries.reverse();

        Feed { entries }
    }
}

impl Entry {
    pub fn play(&self) {
        let _res = Command::new("mpv")
            .arg(&self.link.href)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
}
