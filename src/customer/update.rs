use serde_json::json;

use crate::{Client, Error};

/// Represents a customer to be updated.
pub struct UpdateCustomer<'a> {
    pub email: Option<&'a str>,
    pub username: Option<&'a str>,
}

impl Client {
    /// Update a customer in the ITTV API.
    ///
    /// # Arguments
    ///
    /// * `customer_id` - The ID of the customer to be updated.
    /// * `dto` - The updated customer data.
    ///
    /// # Returns
    ///
    /// If the request is successful, nothing is returned.
    ///
    /// # Errors
    ///
    /// If the request fails, an error is returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ittv_sdk::{Client, UpdateCustomer};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::new("your_api_key");
    ///     
    ///     let customer = UpdateCustomer {
    ///         email: Some("new@email.com"),
    ///         username: None,
    ///     };
    ///
    ///     let response = client
    ///         .update_customer("d6ac4ff04d11", customer)
    ///         .await
    ///         .unwrap();
    ///
    ///     println!("{:?}", response);
    /// }
    /// ```
    pub async fn update_customer(
        &self,
        customer_id: &str,
        dto: UpdateCustomer<'_>,
    ) -> Result<(), Error> {
        let url = format!("{}/alpha/reseller/customer/{}", self.api_url, customer_id);

        let client = reqwest::Client::new();
        let response = client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({ "username": dto.username, "email": dto.email }))
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        let result = response.json::<Error>().await?;
        Err(result)
    }
}
