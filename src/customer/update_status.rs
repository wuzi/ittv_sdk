use serde_json::json;

use crate::{Client, Error};

impl Client {
    /// Update the status of a customer.
    ///
    /// # Arguments
    ///
    /// * `customer_id` - The ID of the customer.
    /// * `status` - The new status of the customer.
    ///
    /// # Returns
    ///
    /// If the request is successful, the function returns Ok. Otherwise, it returns an `Error`.
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
    ///         .update_customer_status("11696d8558ce", "ACTIVE")
    ///         .await
    ///         .unwrap();
    ///
    ///     println!("{:?}", response);
    /// }
    /// ```
    pub async fn update_customer_status(
        &self,
        customer_id: &str,
        status: &str,
    ) -> Result<(), Error> {
        let url = format!(
            "{}/alpha/reseller/customer/{}/status",
            self.api_url, customer_id
        );

        let client = reqwest::Client::new();
        let response = client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({ "status": status }))
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        let result = response.json::<Error>().await?;
        Err(result)
    }
}
