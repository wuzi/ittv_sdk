//! Real socket fixtures for shared connection, deadline and streaming limits.
use super::*;
use std::future::Future;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

struct Server {
    url: String,
    connections: Arc<AtomicUsize>,
    task: JoinHandle<()>,
}
impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl Server {
    async fn start<F, Fut>(handler: F) -> Self
    where
        F: Fn(TcpStream) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let connections = Arc::new(AtomicUsize::new(0));
        let count = connections.clone();
        let task = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                count.fetch_add(1, Ordering::SeqCst);
                tokio::spawn(handler(stream));
            }
        });
        Self {
            url,
            connections,
            task,
        }
    }
}

async fn read_request(stream: &mut TcpStream) -> Option<String> {
    let mut bytes = Vec::new();
    while !bytes.ends_with(b"\r\n\r\n") {
        let mut byte = [0];
        if stream.read_exact(&mut byte).await.is_err() {
            return None;
        }
        bytes.push(byte[0]);
        assert!(bytes.len() < 16 * 1024);
    }
    let headers = String::from_utf8(bytes).unwrap();
    let length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    let mut body = vec![0; length];
    stream.read_exact(&mut body).await.ok()?;
    Some(format!("{headers}{}", String::from_utf8(body).unwrap()))
}

fn short_client(url: &str) -> Client {
    Client::with_policy(
        "secret-key",
        Duration::from_millis(100),
        Duration::from_millis(200),
        1024,
    )
    .with_url(url)
}

#[tokio::test]
async fn total_deadline_covers_slow_headers_stalled_and_continuous_bodies() {
    for behavior in 0..3 {
        let server = Server::start(move |mut stream| async move {
            read_request(&mut stream).await.unwrap();
            if behavior == 0 {
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            if stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n")
                .await
                .is_err()
            {
                return;
            }
            if behavior == 1 {
                let _ = stream.write_all(b"{").await;
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            if behavior == 2 {
                for _ in 0..100 {
                    if stream.write_all(b"x").await.is_err() {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            }
        })
        .await;
        let start = std::time::Instant::now();
        let error = short_client(&server.url)
            .update_customer_status("id", "ACTIVE")
            .await
            .unwrap_err();
        assert!(
            start.elapsed() < Duration::from_secs(1),
            "whole body deadline must terminate"
        );
        assert!(!error.message.contains(&server.url));
        assert_eq!(
            server.connections.load(Ordering::SeqCst),
            1,
            "no replay after uncertain timeout"
        );
    }
}

#[tokio::test]
async fn declared_and_streamed_bodies_are_bounded_for_success_and_failure() {
    for status in [200, 400] {
        for chunked in [false, true] {
            let server = Server::start(move |mut stream| async move {
                read_request(&mut stream).await.unwrap();
                let response = if chunked {
                    format!("HTTP/1.1 {status} response\r\nTransfer-Encoding: chunked\r\n\r\n401\r\n{}\r\n0\r\n\r\n", "x".repeat(1025))
                } else {
                    format!("HTTP/1.1 {status} response\r\nContent-Length: 1025\r\n\r\n")
                };
                let _ = stream.write_all(response.as_bytes()).await;
            }).await;
            let error = short_client(&server.url)
                .update_customer_status("id", "ACTIVE")
                .await
                .unwrap_err();
            assert_eq!(error.message, "ITTV response body exceeds limit");
        }
    }
}

#[tokio::test]
async fn truncated_creation_body_cannot_succeed() {
    let server = Server::start(|mut stream| async move {
        read_request(&mut stream).await.unwrap();
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n{\"_id\":\"id\"}")
            .await
            .unwrap();
    })
    .await;
    assert!(short_client(&server.url)
        .create_customer(new_customer())
        .await
        .is_err());
}

fn new_customer() -> NewCustomer<'static> {
    NewCustomer {
        name: "n",
        cpf: "",
        status: "",
        phone: "",
        email: "",
        username: "",
        password: "secret",
        zipcode: "",
        product: "",
        address: "",
        city: "",
        complement: None,
        number: "",
        state: "",
    }
}

#[tokio::test]
async fn sequential_operations_reuse_one_keepalive_connection() {
    let server = Server::start(|mut stream| async move {
        while read_request(&mut stream).await.is_some() {
            if stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok")
                .await
                .is_err()
            {
                break;
            }
        }
    })
    .await;
    let client = short_client(&server.url);
    client.update_customer_status("id", "ACTIVE").await.unwrap();
    client
        .update_customer_product("id", "product")
        .await
        .unwrap();
    assert_eq!(server.connections.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn concurrent_requests_have_independent_bodies_without_serialization() {
    let (send, mut receive) = mpsc::channel(2);
    let release = Arc::new(tokio::sync::Barrier::new(3));
    let signal = release.clone();
    let server = Server::start(move |mut stream| {
        let send = send.clone();
        let signal = signal.clone();
        async move {
            let request = read_request(&mut stream).await.unwrap();
            send.send(request).await.unwrap();
            signal.wait().await;
            let _ = stream.write_all(b"HTTP/1.1 204 No Content\r\n\r\n").await;
        }
    })
    .await;
    let client = Client::new("secret-key").with_url(&server.url);
    let requests = async {
        tokio::join!(
            client.update_customer_password("one", "password-one"),
            client.update_customer_product("two", "product-two")
        )
    };
    let observe = async {
        let first = receive.recv().await.unwrap();
        let second = receive.recv().await.unwrap();
        for (path, body) in [
            ("/one/changepassword", r#"{"password":"password-one"}"#),
            ("/two/order", r#"{"idProduct":"product-two"}"#),
        ] {
            assert!([&first, &second]
                .iter()
                .any(|request| request.contains(path) && request.ends_with(body)));
        }
        release.wait().await;
    };
    let ((one, two), ()) = tokio::time::timeout(Duration::from_secs(2), async {
        tokio::join!(requests, observe)
    })
    .await
    .expect("requests must reach server concurrently");
    one.unwrap();
    two.unwrap();
}

#[tokio::test]
async fn reqwest_error_conversion_removes_url_and_secrets() {
    let error = reqwest::Client::new()
        .get("http://127.0.0.1:1/private-secret?password=secret")
        .send()
        .await
        .unwrap_err();
    let error = Error::from(error);
    let text = format!("{error:?}");
    assert!(!text.contains("127.0.0.1"));
    assert!(!text.contains("private-secret"));
    assert!(!text.contains("password"));
}

#[tokio::test]
async fn failed_initialization_is_reported_without_unbounded_fallback() {
    let server = Server::start(|_| async {
        panic!("invalid initialization must not send");
    })
    .await;
    let mut client = short_client(&server.url);
    client.http = Err(Error::local("Unable to initialize ITTV transport"));
    let error = client
        .update_customer_status("id", "ACTIVE")
        .await
        .unwrap_err();
    assert_eq!(error.message, "Unable to initialize ITTV transport");
    assert_eq!(server.connections.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn disconnect_after_receiving_mutation_is_not_retried() {
    let server = Server::start(|mut stream| async move {
        read_request(&mut stream).await.unwrap();
    })
    .await;
    assert!(short_client(&server.url)
        .update_customer_status("id", "ACTIVE")
        .await
        .is_err());
    assert_eq!(server.connections.load(Ordering::SeqCst), 1);
}
