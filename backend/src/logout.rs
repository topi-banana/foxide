use axum::extract::Query;
use axum::response::Redirect;
use axum::{Router, routing::get};
use serde::Deserialize;
use tower_cookies::Cookies;

use crate::AppState;

#[derive(Deserialize)]
pub struct LogoutParams {
    redirect: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(logout))
}

async fn logout(Query(params): Query<LogoutParams>, cookies: Cookies) -> Redirect {
    cookies.remove(
        tower_cookies::Cookie::build("auth-session")
            .path("/")
            .build(),
    );
    Redirect::to(params.redirect.as_deref().unwrap_or("/"))
}
