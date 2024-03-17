use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::{ErrorResponse, IttvSdk};

#[derive(Serialize)]
pub struct NewCustomer<'a> {
    pub name: &'a str,
    pub cpf: &'a str,
    pub status: &'a str,
    pub phone: &'a str,
    pub email: &'a str,
    pub username: &'a str,
    pub password: &'a str,
    pub zipcode: &'a str,
    pub product: &'a str,
    pub address: &'a str,
    pub city: &'a str,
    pub complement: Option<&'a str>,
    pub number: &'a str,
    pub state: &'a str,
}

#[derive(Deserialize)]
pub struct CreatedCustomer {
    #[serde(rename = "_id")]
    pub id: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Response {
    CreatedCustomer(CreatedCustomer),
    Error(ErrorResponse),
}

impl IttvSdk {
    pub async fn create_customer(
        &self,
        body: NewCustomer<'_>,
    ) -> Result<CreatedCustomer, Box<dyn Error>> {
        let url = format!("{}/alpha/reseller/customer", self.api_url);
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?
            .json::<Response>()
            .await?;

        match response {
            Response::CreatedCustomer(customer) => Ok(customer),
            Response::Error(e) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.message,
            ))),
        }
    }
}