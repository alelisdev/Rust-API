use crate::apple_api_error_code::AppleApiErrorCode;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Error {
    Unspecified(String),
    ParseError(String),
    SerializationError(String),
    NetworkError(String),
    AppleApiError(AppleApiErrorCode, String),
    GoogleApiError(i32, String),
    InvalidAppleReceipt(String),
    UnexpectedProductId(String),
    SubscriptionNotFound,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Error::Unspecified(g) => g,
            Error::ParseError(g) => g,
            Error::SerializationError(g) => g,
            Error::NetworkError(g) => g,
            Error::AppleApiError(_, g) => g,
            Error::GoogleApiError(_, g) => g,
            Error::InvalidAppleReceipt(g) => g,
            Error::UnexpectedProductId(g) => g,
            Error::SubscriptionNotFound => "Subscription not found.",
        };
        write!(f, "{}", text)
    }
}
