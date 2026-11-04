use ittv_sdk::{Client, NewCustomer};

fn customer() -> NewCustomer<'static> {
    NewCustomer {
        name: "John Doe",
        cpf: "",
        status: "ACTIVE",
        phone: "",
        email: "a@example.com",
        username: "user",
        password: "secret",
        zipcode: "",
        product: "product",
        address: "",
        city: "",
        complement: None,
        number: "",
        state: "",
    }
}

#[tokio::test]
async fn failed_creation_with_id_is_never_success() {
    let mut server = mockito::Server::new_async().await;
    let request = server
        .mock("POST", "/alpha/reseller/customer")
        .match_header("authorization", "Bearer test-key")
        .with_status(409)
        .with_body(r#"{"_id":"not-created","message":"Username já existe."}"#)
        .expect(1)
        .create_async()
        .await;
    let result = Client::new("test-key")
        .with_url(&server.url())
        .create_customer(customer())
        .await;
    request.assert_async().await;
    assert_eq!(
        result
            .expect_err("failed HTTP creation must not yield a customer")
            .message,
        "Username já existe."
    );
}

#[tokio::test]
async fn all_five_wire_contracts_and_base_path() {
    use mockito::Matcher;
    let mut server = mockito::Server::new_async().await;
    let create = server.mock("POST", "/prefix/alpha/reseller/customer")
        .match_header("authorization", "Bearer test-key")
        .match_body(Matcher::Json(serde_json::json!({"name":"John Doe", "cpf":"", "status":"ACTIVE", "phone":"", "email":"a@example.com", "username":"user", "password":"secret", "zipcode":"", "product":"product", "address":"", "city":"", "complement":null, "number":"", "state":""})))
        .with_status(201).with_body(r#"{"_id":"id","additional":"allowed","message":"metadata"}"#).expect(1).create_async().await;
    let update = server
        .mock("PUT", "/prefix/alpha/reseller/customer/id")
        .match_header("authorization", "Bearer test-key")
        .match_body(Matcher::Json(
            serde_json::json!({"username":null,"email":"new@example.com"}),
        ))
        .with_status(204)
        .expect(1)
        .create_async()
        .await;
    let password = server
        .mock("POST", "/prefix/alpha/reseller/customer/id/changepassword")
        .match_header("authorization", "Bearer test-key")
        .match_body(Matcher::Json(serde_json::json!({"password":"new-secret"})))
        .with_status(200)
        .with_body("ordinary success")
        .expect(1)
        .create_async()
        .await;
    let product = server
        .mock("PUT", "/prefix/alpha/reseller/customer/id/order")
        .match_header("authorization", "Bearer test-key")
        .match_body(Matcher::Json(
            serde_json::json!({"idProduct":"new-product"}),
        ))
        .with_status(200)
        .expect(1)
        .create_async()
        .await;
    let status = server
        .mock("PUT", "/prefix/alpha/reseller/customer/id/status")
        .match_header("authorization", "Bearer test-key")
        .match_body(Matcher::Json(serde_json::json!({"status":"INACTIVE"})))
        .with_status(200)
        .expect(1)
        .create_async()
        .await;
    let client = Client::new("test-key").with_url(&format!("{}/prefix", server.url()));
    assert_eq!(client.create_customer(customer()).await.unwrap().id, "id");
    client
        .update_customer(
            "id",
            ittv_sdk::UpdateCustomer {
                email: Some("new@example.com"),
                username: None,
            },
        )
        .await
        .unwrap();
    client
        .update_customer_password("id", "new-secret")
        .await
        .unwrap();
    client
        .update_customer_product("id", "new-product")
        .await
        .unwrap();
    client
        .update_customer_status("id", "INACTIVE")
        .await
        .unwrap();
    for request in [create, update, password, product, status] {
        request.assert_async().await;
    }
}

#[tokio::test]
async fn creation_response_contracts() {
    for (status, body, message) in [
        (
            200,
            r#"{"message":"Username já existe."}"#,
            "Username já existe.",
        ),
        (
            404,
            r#"{"message":"Cliente não encontrado."}"#,
            "Cliente não encontrado.",
        ),
        (
            500,
            r#"{"_id":"must-not-succeed"}"#,
            "ITTV request failed with HTTP status 500",
        ),
        (
            502,
            "private upstream content",
            "ITTV request failed with HTTP status 502",
        ),
        (400, "", "ITTV request failed with HTTP status 400"),
        (200, "", "Invalid ITTV customer response"),
        (200, "{", "Invalid ITTV customer response"),
        (200, r#"{"_id":""}"#, "Invalid ITTV customer response"),
        (200, r#"{"_id":"."}"#, "Invalid ITTV customer response"),
        (200, r#"{"_id":".."}"#, "Invalid ITTV customer response"),
    ] {
        let mut server = mockito::Server::new_async().await;
        let request = server
            .mock("POST", "/alpha/reseller/customer")
            .with_status(status)
            .with_body(body)
            .expect(1)
            .create_async()
            .await;
        let result = Client::new("test-key")
            .with_url(&server.url())
            .create_customer(customer())
            .await;
        assert_eq!(result.unwrap_err().message, message);
        request.assert_async().await;
    }
}

#[tokio::test]
async fn unit_failures_preserve_provider_message_or_safe_status() {
    for (status, body, message) in [
        (
            404,
            r#"{"message":"Cliente não encontrado."}"#,
            "Cliente não encontrado.",
        ),
        (
            502,
            "private upstream content",
            "ITTV request failed with HTTP status 502",
        ),
        (400, "{", "ITTV request failed with HTTP status 400"),
        (503, "", "ITTV request failed with HTTP status 503"),
    ] {
        let mut server = mockito::Server::new_async().await;
        let request = server
            .mock("PUT", "/alpha/reseller/customer/id/status")
            .with_status(status)
            .with_body(body)
            .expect(1)
            .create_async()
            .await;
        let result = Client::new("test-key")
            .with_url(&server.url())
            .update_customer_status("id", "ACTIVE")
            .await;
        assert_eq!(result.unwrap_err().message, message);
        request.assert_async().await;
    }
}

#[tokio::test]
async fn opaque_segments_are_encoded_once_and_dot_segments_rejected_before_io() {
    let mut server = mockito::Server::new_async().await;
    let encoded = server
        .mock("PUT", "/alpha/reseller/customer/a%2Fb%3Fc%23d%252Fe/status")
        .with_status(204)
        .expect(1)
        .create_async()
        .await;
    let dots = server
        .mock("PUT", "/alpha/reseller/customer/.../status")
        .with_status(204)
        .expect(1)
        .create_async()
        .await;
    let client = Client::new("test-key").with_url(&server.url());
    client
        .update_customer_status("a/b?c#d%2Fe", "ACTIVE")
        .await
        .unwrap();
    client
        .update_customer_status("...", "ACTIVE")
        .await
        .unwrap();
    let mut unused = mockito::Server::new_async().await;
    let unexpected = unused
        .mock("PUT", mockito::Matcher::Any)
        .expect(0)
        .create_async()
        .await;
    let client = Client::new("test-key").with_url(&unused.url());
    for id in ["", ".", ".."] {
        assert_eq!(
            client
                .update_customer_status(id, "ACTIVE")
                .await
                .unwrap_err()
                .message,
            "Invalid ITTV customer ID"
        );
    }
    encoded.assert_async().await;
    dots.assert_async().await;
    unexpected.assert_async().await;
}

#[tokio::test]
async fn invalid_base_urls_fail_without_exposing_input() {
    for url in [
        "invalid secret-url",
        "ftp://localhost",
        "http://user:secret@localhost",
        "http://@localhost",
        "http://localhost?private=query",
        "http://localhost#private",
    ] {
        let result = Client::new("secret-key")
            .with_url(url)
            .create_customer(customer())
            .await;
        assert_eq!(result.unwrap_err().message, "Invalid ITTV base URL");
    }
}

#[tokio::test]
async fn redirects_are_not_followed_or_forwarded() {
    let mut destination = mockito::Server::new_async().await;
    let forwarded = destination
        .mock("PUT", mockito::Matcher::Any)
        .expect(0)
        .create_async()
        .await;
    let mut source = mockito::Server::new_async().await;
    let redirect = source
        .mock("PUT", "/alpha/reseller/customer/id/status")
        .with_status(307)
        .with_header("location", &destination.url())
        .expect(1)
        .create_async()
        .await;
    let result = Client::new("secret-key")
        .with_url(&source.url())
        .update_customer_status("id", "ACTIVE")
        .await;
    assert_eq!(
        result.unwrap_err().message,
        "ITTV request failed with HTTP status 307"
    );
    redirect.assert_async().await;
    forwarded.assert_async().await;
}

#[tokio::test]
async fn invalid_authorization_fails_safely_before_io() {
    let mut server = mockito::Server::new_async().await;
    let unexpected = server
        .mock("PUT", mockito::Matcher::Any)
        .expect(0)
        .create_async()
        .await;
    let error = Client::new("private-key\nsecret-password")
        .with_url(&server.url())
        .update_customer_status("id", "ACTIVE")
        .await
        .unwrap_err();
    let diagnostic = format!("{error:?}");
    for secret in ["private-key", "secret-password", &server.url()] {
        assert!(!diagnostic.contains(secret));
    }
    unexpected.assert_async().await;
}
