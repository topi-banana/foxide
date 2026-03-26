use axum::Router;
use axum::routing::get;

async fn login_page() -> &'static str {
    "Login page"
}

pub fn router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new().route("/", get(login_page))
}
