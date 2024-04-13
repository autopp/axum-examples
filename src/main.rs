use axum::body::{Body, Bytes};
use axum::http::request::Parts;
use axum::response::{Json, Response};
use axum::{extract::Request, routing::get, Router};
use serde_json::{json, Value};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = echo_request_router();

    let listener = TcpListener::bind("127.0.0.1:4000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

fn echo_request_router() -> Router {
    // Router::new().route("/echo", get(echo_request))
    Router::new().route("/echo", get(echo_request))
}

async fn echo_request(req: Request) -> Json<Value> {
    Json(json!({
        "method": req.method().as_str(),
        "uri": req.uri().to_string(),
        "headers": req.headers().iter().map(|(k, v)| (k.as_str(), v.to_str().unwrap())).collect::<Vec<_>>(),
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
                .await;

            res.assert_status(200.try_into().unwrap());
            res.assert_json(&json!({}));
        }
    }
}
