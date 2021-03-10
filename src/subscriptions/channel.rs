use crate::errors::Error;
use crate::youtube_feed::{Author, Feed};

use std::cmp::Ordering;

const URL: &str = "https://www.youtube.com/feeds/videos.xml?channel_id=";

/// A single channel with a id and an optional name.
#[derive(Clone, Debug, Hash)]
pub struct Channel {
    id: String,
    pub name: Option<String>,
}

impl PartialEq for Channel {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Channel {}

impl PartialOrd for Channel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Channel {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.name.clone(), other.name.clone()) {
            (Some(sname), Some(oname)) => sname.cmp(&oname),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => self.id.cmp(&other.id),
        }
    }
}

impl Channel {
    /// Create a new channel with just the id.
    pub fn new(id: &str) -> Self {
        Channel {
            id: id.to_string(),
            name: None,
        }
    }

    /// Create a new channel with the id and the name.
    pub fn new_with_name(id: &str, name: &str) -> Self {
        Channel {
            id: id.to_string(),
            name: Some(name.to_string()),
        }
    }

    /// Returns the id of the channel.
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    /// Returns the channel of the channel.
    pub fn get_name(&self) -> Option<String> {
        self.name.clone()
    }

    /// Gets the feed of the channel.
    /// Result in an error if the website of the channel feed could not be loaded.
    /// This may be because there is no internet connection or the channel does not exist.
    /// It will also return an error if the website could not be parsed.
    /// This may be because youtube changed the feed site.
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
                return Err(Error::parsing_website(&self.id));
            }

            // Replace all occurences of `meida:` with `media_` as serde does not seem to like `:`.
            let res3 = res2.unwrap().replace("media:", "media_");

            Ok(res3)
        }
        .await;

        if let Err(e) = content {
            return Err(e);
        }

        let feed_res: Result<Feed, quick_xml::DeError> = quick_xml::de::from_str(&content.unwrap());

        if feed_res.is_err() {
            return Err(Error::parsing_website(&self.id));
        }

        Ok(feed_res.unwrap())
    }
}

impl From<Author> for Channel {
    /// Convert a author to a channel.
    /// Will always set the channel name.
    fn from(author: Author) -> Self {
        let id = author.uri.split("/").last().unwrap();
        let name = author.name;

        Channel::new_with_name(id, &name)
    }
}
