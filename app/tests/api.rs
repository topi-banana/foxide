#![cfg(feature = "ssr")]

use filebrowser_backend::{AppState, build_app};
use leptos::prelude::*;
use reqwest::Client;
use std::net::SocketAddr;

async fn spawn_server() -> SocketAddr {
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let state = AppState::new(
        leptos_options,
        0,
        String::new(),
        0,
        url::Url::parse("http://localhost").unwrap(),
        "",
    );
    let app = build_app(state);

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
        .get(format!("http://{}/health", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}
