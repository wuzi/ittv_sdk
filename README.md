# ITTV SDK

A Rust SDK for interacting with the [ITTV](https://www.ittv.com.br/) API.

## Installation

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
ittv_sdk = "0.1"
```

## Usage

```rust
use ittv_sdk::{Client, NewCustomer};

#[tokio::main]
async fn main() {
    let client = Client::new("your_api_key");
    
    let customer = NewCustomer {
        name: "John Doe",
        // ...
    };

    let response = client
        .create_customer(customer)
        .await
        .unwrap();

    println!("{:?}", response);
}
```
