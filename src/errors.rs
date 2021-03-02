use std::{error, fmt};

#[derive(Clone, Debug)]
pub struct Error {
    error_type: ErrorType,
}

#[derive(Clone, Debug)]
enum ErrorType {
    NetworkError,
    ParseError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.error_type {
            ErrorType::NetworkError => write!(
                f,
                "A network error occured. Are you connected to the internet?"
            ),
            ErrorType::ParseError(channel_id) => write!(
                f,
                "A parse error occured. Is {} a valid channel id?",
                channel_id
            ),
        }
    }
}

impl error::Error for Error {}

impl Error {
    pub fn networking() -> Self {
        Error {
            error_type: ErrorType::NetworkError,
        }
    }

    pub fn parsing(channel_id: &str) -> Self {
        Error {
            error_type: ErrorType::ParseError(channel_id.to_string()),
        }
    }
}
