# ITTV SDK

A Rust SDK for interacting with the [ITTV](https://www.ittv.com.br/) API.

## Installation

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
ittv_sdk = "0.2"
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

Check the [documentation](https://docs.rs/ittv_sdk/latest/ittv_sdk/struct.Client.html) for more examples.

## Transport behavior

A `Client` keeps one reusable HTTP connection pool. Each request has a 5-second
connection timeout and a 30-second total deadline, including response-body reads.
Redirects and automatic retries are disabled. A timeout or disconnected response
can leave a mutation's completion uncertain; reconcile with the provider before
submitting it again.
Creation also rejects a successful response whose customer ID is empty, `.` or
`..`. A rejected response may still mean the provider created a customer;
reconcile externally before retrying creation.

Successful and failed response bodies are limited to 1 MiB (1,048,576 bytes),
checked both against declared length and while reading streamed bodies. The SDK
supports a customer ID response and provider message responses, with no list API;
this ceiling is generous for those shapes and the checked local fixtures. Unit
updates accept ordinary 2xx responses, including 204/empty bodies, under the same
size and deadline limits. Creation requires a successful HTTP status and a valid
customer response; a failed status cannot become success merely because its body
contains an `_id`. Existing provider error messages are preserved.

`Client::new` and `with_url` remain infallible. Initialization or base URL errors
are retained and reported by operations without sending a request or falling back
to an unbounded transport. `with_url` accepts HTTP/HTTPS bases with an optional
base path and trailing slash; operation paths are appended to that base path.
URL credentials, query strings and fragments are rejected. Customer IDs are opaque
single path segments: reserved characters and percent signs are encoded once;
empty IDs, `.` and `..` are rejected before sending. Values such as `...` remain
valid opaque IDs. Transport errors omit the request URL. The SDK does not add API keys or request
payloads to diagnostic text; provider error messages are retained verbatim.

## Migrating from 0.1

Version 0.2 moves from reqwest 0.11 to reqwest 0.12 (at least 0.12.28), retaining
default native TLS and HTTP/2. The public `From<reqwest::Error>` conversion now
accepts **reqwest 0.12** errors. Callers that convert their own reqwest 0.11 errors
must migrate that dependency or handle the conversion themselves. The SDK's
`Client::new`, `with_url`, five customer operations, DTO fields and public
`Error { message }` shape remain available. No legacy reqwest stack is retained.

Before releasing, validate the reviewed source and lockfile with tests, doctests,
Clippy, formatting, a frozen RustSec audit, package verification and an exact
package consumer check. API adoption must explicitly select `ittv_sdk = "0.2"`;
local source edits do not update a registry consumer. Publication, package
provenance and consumer adoption are separate release gates.
