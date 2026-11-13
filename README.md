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
use ittv_sdk::Client;

let client = Client::new("your_api_key");
```

Use `NewCustomer` with `Client::create_customer` and `UpdateCustomer` with
`Client::update_customer`; password, product and status updates have dedicated
methods. See the [client reference](https://docs.rs/ittv_sdk/latest/ittv_sdk/struct.Client.html)
and the [complete creation example](src/customer/create.rs) for required fields.

## Transport behavior

A client reuses one connection pool, with a 5-second connect timeout, 30-second
request deadline (including body reads), and 1 MiB response limit. Redirects and
automatic retries are disabled. Reconcile an uncertain mutation with the provider
before resubmitting it; a timeout does not prove failure.

`with_url` accepts an HTTP(S) base with an optional path; operation paths append
to it. Credentials, queries and fragments in the base are rejected. Customer IDs
are opaque path segments; empty IDs and literal `.`/`..` are invalid. Constructor
or base URL errors surface on operations before dispatch. Provider error messages
are retained verbatim, so apply the caller's disclosure policy.

## Migrating from 0.1

Version 0.2 moves from reqwest 0.11 to reqwest 0.12 (at least 0.12.28), retaining
default native TLS and HTTP/2. The public `From<reqwest::Error>` conversion now
accepts **reqwest 0.12** errors. Callers that convert their own reqwest 0.11 errors
must migrate that dependency or handle the conversion themselves. The SDK's
`Client::new`, `with_url`, five customer operations, DTO fields and public
`Error { message }` shape remain available. No legacy reqwest stack is retained.

API adoption must explicitly select `ittv_sdk = "0.2"`; editing this checkout does
not update a registry consumer. Publication and downstream adoption are separate
release steps.
