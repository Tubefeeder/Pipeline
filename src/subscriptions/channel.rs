use crate::errors::Error;
use crate::youtube_feed::{Author, Feed};

use std::cmp::Ordering;

use regex::Regex;
use std::hash::{Hash, Hasher};

const FEED_URL: &str = "https://www.youtube.com/feeds/videos.xml?channel_id=";

/// A single channel with a id and an optional name.
#[derive(Clone, Debug)]
pub struct Channel {
    id: String,
    pub name: Option<String>,
}

impl PartialEq for Channel {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Channel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
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
            (Some(sname), Some(oname)) => sname.to_uppercase().cmp(&oname.to_uppercase()),
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

    /// Try to create a channel from a given `&str`, that may be a id or name.
    /// It first tries to check if it is a valid id, otherwise it will try to interpret
    /// it as a channel name.
    #[tokio::main]
    pub async fn from_id_or_name(id_or_name: &str) -> Result<Self, Error> {
        let from_id = Channel::new(id_or_name);
        if from_id.get_feed_no_main().await.is_ok() {
            Ok(from_id)
        } else {
            Channel::from_name(id_or_name).await
        }
    }

    /// Cretes a new channel from the given name.
    /// Will try to download the channels youtube page and get the id.
    async fn from_name(name: &str) -> Result<Self, Error> {
        let url = format!("https://www.youtube.com/c/{}/featured", name);
        let content: Result<String, Error> = async {
            let response = reqwest::get(&url).await;

            if response.is_err() {
                return Err(Error::networking());
            }

            let parsed = response.unwrap().text().await;

            if parsed.is_err() {
                return Err(Error::parsing_website(name));
            }

            Ok(parsed.unwrap())
        }
        .await;

        if let Err(e) = content {
            Err(e)
        } else {
            let regex = Regex::new(r#""externalId":"([0-9a-zA-Z_\-]*)"#).unwrap();

            if let Some(id) = regex.captures(&content.unwrap()) {
                Ok(Channel::new_with_name(&id[1].to_string(), name))
            } else {
                Err(Error::parsing_website(name))
            }
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
        self.get_feed_no_main().await
    }

    /// Get the feed without needing `#[tokio::main]`.
    async fn get_feed_no_main(&self) -> Result<Feed, Error> {
        let url = FEED_URL.to_string() + &self.id;

        let content: Result<String, Error> = async {
            let res1 = reqwest::get(&url).await;

            if res1.is_err() {
                return Err(Error::networking());
            }

            let res2 = res1.unwrap().text().await;

            if res2.is_err() {
                return Err(Error::parsing_website(&self.id));
            }

            // Replace all occurences of `media:` with `media_` as serde does not seem to like `:`.
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
        let id = author.uri.split('/').last().unwrap();
        let name = author.name;

        Channel::new_with_name(id, &name)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_order() {
        assert!(Channel::new("abcdef") < Channel::new("ghijkl"));
        assert!(Channel::new("abcdef") > Channel::new_with_name("ghijkl", "z"));
        assert!(Channel::new_with_name("abcdef", "z") > Channel::new_with_name("ghijkl", "a"));
    }

    #[tokio::test]
    async fn test_get_id_from_name() {
        let name = "Brodie Robertson";

        let from_name = Channel::from_name(name).await;

        assert!(from_name.is_ok());

        assert_eq!(
            from_name.unwrap(),
            Channel::new_with_name("UCld68syR8Wi-GY_n4CaoJGA", name)
        );
    }

    #[tokio::test]
    async fn test_get_id_invalid_name() {
        let name = "I hope nobody will ever call their youtube-channel this name";

        let from_name = Channel::from_name(name).await;

        assert!(from_name.is_err());
    }
}
