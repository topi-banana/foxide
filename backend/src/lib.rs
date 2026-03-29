mod health;
mod login;

use std::sync::Arc;

use axum::Router;
use axum::extract::FromRef;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::trace::TraceLayer;

use filebrowser_frontend::{App, shell};

use crate::login::{TokenStorage, UserStorage};

/// Application-wide shared state.
#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,

    pub client_id: u64,
    pub client_secret: String,
    pub guild_id: u64,
    pub redirect_uri: url::Url,

    pub token_storage: Arc<TokenStorage>,
    pub user_storage: Arc<UserStorage>,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

pub async fn run() {
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let client_id: u64 = std::env::var("CLIENT_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| {
            tracing::error!("CLIENT_ID is not set or invalid, using default 0");
            0
        });
    let client_secret = std::env::var("CLIENT_SECRET").unwrap_or_else(|_| {
        tracing::error!("CLIENT_SECRET is not set, using empty string");
        String::new()
    });
    let guild_id: u64 = std::env::var("GUILD_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| {
            tracing::error!("GUILD_ID is not set or invalid, using default 0");
            0
        });
    let redirect_uri = std::env::var("REDIRECT_URI")
        .ok()
        .and_then(|v| v.parse::<url::Url>().ok())
        .unwrap_or_else(|| {
            let default = format!("http://{}", addr);
            tracing::error!("REDIRECT_URI is not set or invalid, using {default}");
            url::Url::parse(&default).unwrap()
        });

    let state = AppState {
        leptos_options,
        client_id,
        client_secret,
        guild_id,
        redirect_uri,
        token_storage: Arc::new(TokenStorage::new()),
        user_storage: Arc::new(UserStorage::new()),
    };
    let app = build_app(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

/// Build the full application router with API routes and Leptos SSR.
pub fn build_app(state: AppState) -> Router {
    let routes = generate_route_list(App);

    Router::<AppState>::new()
        .nest("/health", crate::health::router())
        .nest("/login", crate::login::router())
        .leptos_routes(&state, routes, {
            let options = state.leptos_options.clone();
            move || shell(options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
