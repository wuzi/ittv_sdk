//! # ITTV SDK
//!
//! A Rust SDK for the ITTV API. Requests share a connection pool, have a
//! five-second connection timeout and a thirty-second total deadline (including
//! the response body), and retain at most 1 MiB of response data. Redirects and
//! automatic retries are disabled. A mutation that times out may have completed
//! at the provider; callers must reconcile its outcome before trying again.

use std::time::Duration;

use reqwest::{Method, Url};
use serde::{Deserialize, Serialize};

pub use crate::customer::NewCustomer;
pub use crate::customer::UpdateCustomer;

mod customer;
#[cfg(test)]
mod transport_tests;

pub(crate) fn usable_customer_id(id: &str) -> bool {
    !matches!(id, "" | "." | "..")
}

/// An ITTV API client with one shared, bounded HTTP connection pool.
///
/// Initialization and base URL validation errors are retained and returned by
/// operations, preserving the infallible constructor without an unbounded fallback.
pub struct Client {
    api_key: String,
    base_url: Result<Url, Error>,
    http: Result<reqwest::Client, Error>,
    body_limit: usize,
}

impl Client {
    /// Create a client with a five-second connect and thirty-second total timeout.
    #[must_use]
    pub fn new(api_key: &str) -> Self {
        Self::with_policy(
            api_key,
            Duration::from_secs(5),
            Duration::from_secs(30),
            1024 * 1024,
        )
    }

    fn with_policy(api_key: &str, connect: Duration, total: Duration, body_limit: usize) -> Self {
        let http = reqwest::Client::builder()
            .connect_timeout(connect)
            .timeout(total)
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .build()
            .map_err(|_| Error::local("Unable to initialize ITTV transport"));
        Self {
            api_key: api_key.to_owned(),
            base_url: Self::parse_base("https://api-resellers.ittv.com.br"),
            http,
            body_limit,
        }
    }

    /// Override the HTTP/HTTPS base URL, optionally including a base path.
    ///
    /// Endpoint paths are appended after that base path. A trailing slash is
    /// accepted. Credentials, query strings and fragments are rejected; invalid
    /// bases produce an operation error before any request is sent.
    pub fn with_url(mut self, api_url: &str) -> Self {
        self.base_url = Self::parse_base(api_url);
        self
    }

    fn parse_base(value: &str) -> Result<Url, Error> {
        let invalid = || Error::local("Invalid ITTV base URL");
        let url = Url::parse(value).map_err(|_| invalid())?;
        // Check the original authority too: URL normalization can erase an empty
        // user-info component (`http://@host`). Never silently accept credentials.
        let (_, rest) = value.split_once("://").ok_or_else(invalid)?;
        let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || authority.is_empty()
            || authority.contains('@')
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(invalid());
        }
        Ok(url)
    }

    async fn send<T: Serialize + ?Sized>(
        &self,
        method: Method,
        customer_id: Option<&str>,
        suffix: Option<&str>,
        body: &T,
    ) -> Result<Vec<u8>, Error> {
        let mut url = self
            .base_url
            .as_ref()
            .map_err(|error| Error::local(&error.message))?
            .clone();
        if customer_id.is_some_and(|id| !usable_customer_id(id)) {
            return Err(Error::local("Invalid ITTV customer ID"));
        }
        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| Error::local("Invalid ITTV base URL"))?;
            path.pop_if_empty()
                .extend(["alpha", "reseller", "customer"]);
            if let Some(id) = customer_id {
                path.push(id);
            }
            if let Some(suffix) = suffix {
                path.push(suffix);
            }
        }
        let http = self
            .http
            .as_ref()
            .map_err(|error| Error::local(&error.message))?;
        let mut response = http
            .request(method, url)
            .bearer_auth(&self.api_key)
            .json(body)
            .send()
            .await?;
        let status = response.status();
        if response
            .content_length()
            .is_some_and(|length| length > self.body_limit as u64)
        {
            return Err(Error::local("ITTV response body exceeds limit"));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if chunk.len() > self.body_limit - bytes.len() {
                return Err(Error::local("ITTV response body exceeds limit"));
            }
            bytes.extend_from_slice(&chunk);
        }
        if !status.is_success() {
            return Err(
                serde_json::from_slice::<Error>(&bytes).unwrap_or_else(|_| Error {
                    message: format!("ITTV request failed with HTTP status {}", status.as_u16()),
                }),
            );
        }
        Ok(bytes)
    }

    async fn send_unit<T: Serialize + ?Sized>(
        &self,
        method: Method,
        customer_id: &str,
        suffix: Option<&str>,
        body: &T,
    ) -> Result<(), Error> {
        self.send(method, Some(customer_id), suffix, body).await?;
        Ok(())
    }
}

/// Generic error response from the ITTV API.
#[derive(Debug, Deserialize)]
pub struct Error {
    pub message: String,
}

impl Error {
    fn local(message: &str) -> Self {
        Self {
            message: message.to_owned(),
        }
    }
}

/// Converts reqwest 0.12 errors, removing their request URL from diagnostics.
impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self {
            message: error.without_url().to_string(),
        }
    }
}
