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
 * along with Foobar.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use std::{error, fmt};

#[derive(Clone, Debug)]
pub struct Error {
    error_type: ErrorType,
}

#[derive(Clone, Debug)]
enum ErrorType {
    Network,
    ParseWebsite(String),
    ParseSubscriptions(String),
    GeneralSubscriptions(String, String),
    ParseFeed(String),
    GeneralFeed(String, String),
    ParseFilter(String),
    GeneralFilter(String, String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.error_type {
            ErrorType::Network => write!(
                f,
                "A network error occured. Are you connected to the internet?"
            ),
            ErrorType::ParseWebsite(channel_id) => write!(
                f,
                "Could not parse feed of channel. Is {} a valid channel id or name?",
                channel_id
            ),
            ErrorType::ParseSubscriptions(subscriptions_file) => write!(
                f,
                "Could not parse subscriptions. Check the construction of {}.",
                subscriptions_file
            ),
            ErrorType::GeneralSubscriptions(error_type, subscriptions_file) => write!(
                f,
                "Error {} the subscription file {}.",
                error_type, subscriptions_file
            ),
            ErrorType::ParseFeed(feed_file) => write!(
                f,
                "Could not parse feed. Check the construction of {}.",
                feed_file
            ),
            ErrorType::GeneralFeed(error_type, feed_file) => write!(
                f,
                "Error {} the subscription file {}.",
                error_type, feed_file
            ),
            ErrorType::ParseFilter(regex) => {
                write!(f, "Could not parse the regex {}.", regex)
            }
            ErrorType::GeneralFilter(error_type, filter_file) => {
                write!(f, "Error {} the filter file {}.", error_type, filter_file)
            }
        }
    }
}

impl error::Error for Error {}

impl Error {
    pub fn networking() -> Self {
        Error {
            error_type: ErrorType::Network,
        }
    }

    pub fn parsing_website(channel_id: &str) -> Self {
        Error {
            error_type: ErrorType::ParseWebsite(channel_id.to_string()),
        }
    }

    pub fn parsing_subscriptions(subscriptions_file: &str) -> Self {
        Error {
            error_type: ErrorType::ParseSubscriptions(subscriptions_file.to_string()),
        }
    }

    pub fn general_subscriptions(error_type: &str, subscriptions_file: &str) -> Self {
        Error {
            error_type: ErrorType::GeneralSubscriptions(
                error_type.to_string(),
                subscriptions_file.to_string(),
            ),
        }
    }

    pub fn parsing_filter(filter: &str) -> Self {
        Error {
            error_type: ErrorType::ParseFilter(filter.to_string()),
        }
    }

    pub fn general_filter(error_type: &str, filter_file: &str) -> Self {
        Error {
            error_type: ErrorType::GeneralFilter(error_type.to_string(), filter_file.to_string()),
        }
    }

    pub fn parsing_feed(feed_file: &str) -> Self {
        Error {
            error_type: ErrorType::ParseFeed(feed_file.to_string()),
        }
    }

    pub fn general_feed(error_type: &str, feed_file: &str) -> Self {
        Error {
            error_type: ErrorType::GeneralFeed(error_type.to_string(), feed_file.to_string()),
        }
    }
}
