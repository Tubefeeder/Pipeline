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
                "Could not parse feed of channel. Is {} a valid channel id?",
                channel_id
            ),
            ErrorType::ParseSubscriptions(subscriptions_file) => write!(
                f,
                "Could not parse subscriptions. Check the construction of {}",
                subscriptions_file
            ),
            ErrorType::GeneralSubscriptions(error_type, subscriptions_file) => write!(
                f,
                "Error {} the subscription file {}",
                error_type, subscriptions_file
            ),
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
}
