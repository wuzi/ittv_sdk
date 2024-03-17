use serde_json::json;

use crate::{Client, Error};

impl Client {
    /// Update a customer's password.
    ///
    /// # Arguments
    /// * `customer_id` - The ID of the customer to update.
    /// * `password` - The new password for the customer.
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
    /// use ittv_sdk::{Client};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::new("your_api_key");
    ///     
    ///     let response = client
    ///         .update_customer_password("11696d8558ce", "pa$$w0rd!")
    ///         .await
    ///         .unwrap();
    ///
    ///     println!("{:?}", response);
    /// }
    /// ```
    pub async fn update_customer_password(
        &self,
        customer_id: &str,
        password: &str,
    ) -> Result<(), Error> {
        let url = format!(
            "{}/alpha/reseller/customer/{}/changepassword",
            self.api_url, customer_id
        );

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({ "password": password }))
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        let result = response.json::<Error>().await?;
        Err(result)
    }
}
