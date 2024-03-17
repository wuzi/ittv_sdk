use serde::Deserialize;

mod customer;

pub struct IttvSdk {
    api_key: String,
    api_url: String,
}

impl IttvSdk {
    #[must_use]
    pub fn new(api_key: &str, api_url: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            api_url: api_url.to_string(),
        }
    }
}

#[derive(Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}
