use std::io::ErrorKind;

use axum::body::Body;

use axum::http::{HeaderMap, Method, Uri};
use axum::response::Json;
use axum::routing::post;
use axum::{routing::get, Router};
use futures::TryStreamExt;
use serde_json::{json, Value};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio_util::io::StreamReader;

#[tokio::main]
async fn main() {
    let app = echo_request_router()
        .merge(echo_request_body_router())
        .merge(echo_full_router());

    let listener = TcpListener::bind("127.0.0.1:4000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

fn echo_request_router() -> Router {
    // Router::new().route("/echo", get(echo_request))
    Router::new().route("/echo", get(echo_request))
}

async fn echo_request(method: Method, uri: Uri, headers: HeaderMap) -> Json<Value> {
    Json(json!({
        "method": method.as_str(),
        "uri": uri.to_string(),
        "headers": headers.iter().map(|(k, v)| (k.as_str(), v.to_str().unwrap())).collect::<Vec<_>>(),
    }))
}

fn echo_request_body_router() -> Router {
    // Router::new().route("/echo", get(echo_request))
    Router::new().route("/echo_body", post(echo_request_body))
}

async fn echo_request_body(body: Body) -> Json<Value> {
    let mut stream = StreamReader::new(
        body.into_data_stream()
            .map_err(|err| std::io::Error::new(ErrorKind::Other, err)),
    );

    let mut buf: Vec<u8> = vec![];
    stream.read_buf(&mut buf).await.unwrap();

    Json(json!({
        "body": String::from_utf8_lossy(&buf),
    }))
}

fn echo_full_router() -> Router {
    // Router::new().route("/echo", get(echo_request))
    Router::new().route("/echo_full", post(echo_full))
}

async fn echo_full(method: Method, uri: Uri, headers: HeaderMap, body: Body) -> Json<Value> {
    let mut stream = StreamReader::new(
        body.into_data_stream()
            .map_err(|err| std::io::Error::new(ErrorKind::Other, err)),
    );

    let mut buf: Vec<u8> = vec![];
    stream.read_buf(&mut buf).await.unwrap();

    Json(json!({
        "method": method.as_str(),
        "uri": uri.to_string(),
        "headers": headers.iter().map(|(k, v)| (k.as_str(), v.to_str().unwrap())).collect::<Vec<_>>(),
        "body": String::from_utf8_lossy(&buf),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    mod echo_request {
        use super::*;

        #[tokio::test]
        async fn it_returns_request_info() {
            let app = echo_request_router();
            let server = TestServer::new(app).unwrap();

            let res = server
                .get("/echo")
                .add_header("foo".try_into().unwrap(), "bar".try_into().unwrap())
                .add_header("answer".try_into().unwrap(), "42".try_into().unwrap())
                .await;

            res.assert_status(200.try_into().unwrap());
            res.assert_json(&json!({
                "method": "GET",
                "uri": "http://localhost/echo",
                "headers": [["foo", "bar"], ["answer", "42"]],
            }));
        }
    }

    mod echo_request_body {
        use super::*;

        #[tokio::test]
        async fn it_returns_request_body() {
            let app = echo_request_body_router();
            let server = TestServer::new(app).unwrap();

            let res = server
                .post("/echo_body")
                .json(&json!({ "body": "hello world" }))
                .await;

            res.assert_status(200.try_into().unwrap());
            res.assert_json(&json!({
                "body": r#"{"body":"hello world"}"#,
            }));
        }
    }

    mod echo_full {
        use super::*;

        #[tokio::test]
        async fn it_returns_request_body() {
            let app = echo_full_router();
            let server = TestServer::new(app).unwrap();

            let res = server
                .post("/echo_full")
                .add_header("foo".try_into().unwrap(), "bar".try_into().unwrap())
                .json(&json!({ "body": "hello world" }))
                .await;

            res.assert_status(200.try_into().unwrap());
            res.assert_json(&json!({
                "method": "POST",
                "uri": "http://localhost/echo_full",
                "headers": [["content-type", "application/json"], ["foo", "bar"]],
                "body": r#"{"body":"hello world"}"#,
            }));
        }
    }
}
