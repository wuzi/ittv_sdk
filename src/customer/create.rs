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
        let bytes = self.send(reqwest::Method::POST, None, None, &body).await?;
        let response = serde_json::from_slice::<Response>(&bytes)
            .map_err(|_| Error::local("Invalid ITTV customer response"))?;

        match response {
            Response::Customer(customer) if crate::usable_customer_id(&customer.id) => Ok(customer),
            Response::Customer(_) => Err(Error::local("Invalid ITTV customer response")),
            Response::Error(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_customer() {
        let client = Client::new("your_api_key");
        let mut ittv = mockito::Server::new_async().await;

        ittv.mock("POST", "/alpha/reseller/customer")
            .match_header("Authorization", "Bearer your_api_key")
            .with_status(200)
            .with_body(r#"{"_id":"5f5e7e3b3f3b3b3b3b3b3b3b"}"#)
            .create();

        let customer = NewCustomer {
            name: "John Doe",
            cpf: "",
            status: "",
            phone: "",
            email: "",
            username: "",
            password: "",
            zipcode: "",
            product: "",
            address: "",
            city: "",
            complement: None,
            number: "",
            state: "",
        };

        let response = client
            .with_url(ittv.url().as_str())
            .create_customer(customer)
            .await
            .unwrap();

        assert_eq!(response.id, "5f5e7e3b3f3b3b3b3b3b3b3b");
    }
}
