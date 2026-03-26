use axum::{Json, Router, routing::get};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub fn router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new().route("/", get(health))
}
