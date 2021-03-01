use crate::errors::Error;
use crate::youtube_feed::feed::{Author, Feed};

const URL: &str = "https://www.youtube.com/feeds/videos.xml?channel_id=";

#[derive(PartialEq, Eq, Clone)]
pub struct Channel {
    id: String,
}

impl Channel {
    pub fn new(id: &str) -> Self {
        Channel { id: id.to_string() }
    }

    pub fn get_feed(&self) -> Result<Feed, Error> {
        let url = URL.to_string() + &self.id;

        let content: Result<String, Error> = async_std::task::block_on(async {
            let res1 = reqwest::get(&url).await;

            if res1.is_err() {
                return Err(Error::networking());
            }

            let res2 = res1.unwrap().text().await;

            if res2.is_err() {
                return Err(Error::parsing(&self.id));
            }

            Ok(res2.unwrap())
        });

        if let Err(e) = content {
            return Err(e);
        }

        let feed_res: Result<Feed, quick_xml::DeError> = quick_xml::de::from_str(&content.unwrap());

        if feed_res.is_err() {
            return Err(Error::parsing(&self.id));
        }

        Ok(feed_res.unwrap())
    }
}

impl From<Author> for Channel {
    fn from(author: Author) -> Self {
        let id = author.uri.rsplit("/").last().unwrap();

        Channel::new(id)
    }
}

#[derive(Clone)]
pub struct ChannelGroup {
    channels: Vec<Channel>,
}

impl ChannelGroup {
    pub fn new() -> Self {
        ChannelGroup {
            channels: Vec::new(),
        }
    }

    pub fn add(&mut self, channel: Channel) {
        if !self.channels.contains(&channel) {
            self.channels.push(channel);
        }
    }

    pub fn get_feed(&self) -> Result<Feed, Error> {
        let mut feeds: Vec<Feed> = Vec::new();

        for channel in &self.channels {
            let feed_res = channel.get_feed();
            if let Err(e) = feed_res {
                return Err(e);
            }
            feeds.push(feed_res.unwrap());
        }

        Ok(Feed::combine(feeds))
    }
}
