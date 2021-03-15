use crate::errors::Error;
use crate::youtube_feed::Entry;

use regex::Regex;

/// Filter for feed entries.
#[derive(Clone, Debug)]
pub struct EntryFilter {
    /// The filter for the title.
    title_filter: Regex,
    /// The filter for the channel.
    channel_filter: Regex,
}

impl PartialEq for EntryFilter {
    fn eq(&self, other: &Self) -> bool {
        self.get_title_filter_string() == other.get_title_filter_string()
            && self.get_channel_filter_string() == other.get_channel_filter_string()
    }
}

impl Eq for EntryFilter {}

impl EntryFilter {
    /// Create a new
    pub fn new(title_filter: &str, channel_filter: &str) -> Result<Self, Error> {
        let title_reg = Regex::new(title_filter);
        if title_reg.is_err() {
            return Err(Error::parsing_filter(title_filter));
        }

        let channel_reg = Regex::new(channel_filter);
        if channel_reg.is_err() {
            return Err(Error::parsing_filter(channel_filter));
        }

        Ok(EntryFilter {
            title_filter: title_reg.unwrap(),
            channel_filter: channel_reg.unwrap(),
        })
    }

    /// Get the title filter as a string.
    pub fn get_title_filter_string(&self) -> String {
        self.title_filter.clone().to_string()
    }

    /// Get the channel filter as a string.
    pub fn get_channel_filter_string(&self) -> String {
        self.channel_filter.clone().to_string()
    }

    /// Test if filter matches. A filter matches if both the title and channel matches.
    pub fn matches(&self, entry: &Entry) -> bool {
        self.title_filter.is_match(&entry.title) && self.channel_filter.is_match(&entry.author.name)
    }
}
