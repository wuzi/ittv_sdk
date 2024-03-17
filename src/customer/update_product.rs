use serde_json::json;

use crate::{Client, Error};

impl Client {
    /// Update a customer's product.
    ///
    /// # Arguments
    ///
    /// * `customer_id` - The ID of the customer to update.
    /// * `product_id` - The new product for the customer.
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
    ///         .update_customer_product("11696d8558ce", "c2293a8eee5c")
    ///         .await
    ///         .unwrap();
    ///
    ///     println!("{:?}", response);
    /// }
    /// ```
    pub async fn update_customer_product(
        &self,
        customer_id: &str,
        product_id: &str,
    ) -> Result<(), Error> {
        let url = format!(
            "{}/alpha/reseller/customer/{}/order",
            self.api_url, customer_id
        );

        let client = reqwest::Client::new();
        let response = client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({ "idProduct": product_id }))
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        let result = response.json::<Error>().await?;
        Err(result)
    }
}
