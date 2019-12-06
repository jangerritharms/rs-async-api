use core::option::NoneError;
use std::fmt;
use std::error;

#[derive(Debug, PartialEq)]
pub enum Error {
    KrakenError(Vec<String>),
    APIError(String),
    JSONError(String),
    StreamError,
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::APIError(err.to_string())
    }
}

impl From<Error> for NoneError {
    fn from(err: Error) -> Self {
        NoneError
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JSONError(err.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Library error")
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
