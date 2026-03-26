use axum::Router;
use axum::routing::get;
use reqwest::Client;
use std::net::SocketAddr;

async fn spawn_server() -> SocketAddr {
    let app = Router::new().route("/api/health", get(filebrowser_backend::api::health));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });

    addr
}

#[tokio::test]
async fn health_returns_ok() {
    let addr = spawn_server().await;
    let client = Client::new();

    let resp = client
        .get(format!("http://{}/api/health", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}
