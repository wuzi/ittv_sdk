use serde::{Deserialize, Serialize};

use crate::{Client, Error};

/// Represents a new customer to be created.
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

/// Represents a customer created in the ITTV API.
#[derive(Debug, Deserialize)]
pub struct Customer {
    #[serde(rename = "_id")]
    pub id: String,
}

/// Represents the response from the ITTV API when creating a customer.
#[derive(Deserialize)]
#[serde(untagged)]
enum Response {
    Customer(Customer),
    Error(Error),
}

impl Client {
    /// Create a new customer in the ITTV API.
    ///
    /// # Arguments
    ///
    /// * `body` - The new customer to be created.
    ///
    /// # Returns
    ///
    /// The created customer.
    ///
    /// # Errors
    ///
    /// If the request fails, an error is returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ittv_sdk::{Client, NewCustomer};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::new("your_api_key");
    ///     
    ///     let customer = NewCustomer {
    ///         name: "John Doe",
    ///         cpf: "",
    ///         status: "",
    ///         phone: "",
    ///         email: "",
    ///         username: "",
    ///         password: "",
    ///         zipcode: "",
    ///         product: "",
    ///         address: "",
    ///         city: "",
    ///         complement: None,
    ///         number: "",
    ///         state: ""
    ///     };
    ///
    ///     let response = client
    ///         .create_customer(customer)
    ///         .await
    ///         .unwrap();
    ///
    ///     println!("{:?}", response);
    /// }
    /// ```
    pub async fn create_customer(&self, body: NewCustomer<'_>) -> Result<Customer, Error> {
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
            Response::Customer(customer) => Ok(customer),
            Response::Error(e) => Err(e),
        }
    }
}
