//! # ITTV SDK
//!
//! A Rust SDK for the ITTV API.

use serde::Deserialize;

pub use crate::customer::NewCustomer;
pub use crate::customer::UpdateCustomer;

mod customer;

pub struct Client {
    api_key: String,
    api_url: String,
}

impl Client {
    /// Create a new instance of the ITTV SDK.
    #[must_use]
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            api_url: "https://api-resellers.ittv.com.br".to_string(),
        }
    }
}

/// Generic error response from the ITTV API.
#[derive(Debug, Deserialize)]
pub struct Error {
    pub message: String,
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}
