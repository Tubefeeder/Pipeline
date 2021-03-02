use crate::errors::Error;
use crate::youtube_feed::feed::{Author, Feed};

use rayon::prelude::*;

use futures;

const URL: &str = "https://www.youtube.com/feeds/videos.xml?channel_id=";

#[derive(PartialEq, Eq, Clone)]
pub struct Channel {
    id: String,
}

impl Channel {
    pub fn new(id: &str) -> Self {
        Channel { id: id.to_string() }
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

            Ok(res2.unwrap())
        }
        .await;

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

    pub async fn get_feed(&self) -> Result<Feed, Error> {
        // for channel in &self.channels {
        //     let channel_feed = channel.get_feed();
        //     if let Err(e) = channel_feed {
        //         return Err(e);
        //     } else {
        //         feeds.push(channel_feed.unwrap());
        //         // let rnd_delta: i64 = rng.gen_range(-250..250);
        //         //     thread::sleep(time::Duration::from_millis(
        //         //         (500 + rnd_delta).try_into().unwrap(),
        //         //     ));
        //     }
        // }

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
