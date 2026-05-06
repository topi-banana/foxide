mod admin;
mod health;
mod login;
mod logout;
pub mod volume;
mod ws;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use serenity::http::Http;
use tower_cookies::CookieManagerLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::admin::AdminSettings;
use crate::login::{TokenStorage, UserStorage};
use crate::volume::VolumeStorage;
use crate::ws::SocketStorage;

const DEFAULT_SITE_ADDR: &str = "127.0.0.1:3000";
const DEFAULT_DIST_DIR: &str = "dist";

/// Application-wide shared state.
#[derive(Clone)]
pub struct AppState {
    pub site_addr: SocketAddr,

    pub client_id: u64,
    pub client_secret: String,
    pub guild_id: u64,
    pub redirect_uri: url::Url,

    pub bot_http: Arc<Http>,
    pub admin_settings: Arc<AdminSettings>,
    pub socket_storage: Arc<SocketStorage>,
    pub token_storage: Arc<TokenStorage>,
    pub user_storage: Arc<UserStorage>,
    pub volume_storage: Arc<VolumeStorage>,
}

impl AppState {
    pub fn new(
        site_addr: SocketAddr,
        client_id: u64,
        client_secret: String,
        guild_id: u64,
        redirect_uri: url::Url,
        bot_token: &str,
    ) -> Self {
        Self {
            site_addr,
            client_id,
            client_secret,
            guild_id,
            redirect_uri,
            bot_http: Arc::new(Http::new(bot_token)),
            admin_settings: Arc::new(
                AdminSettings::open("data/admin").expect("failed to open admin settings"),
            ),
            socket_storage: Arc::new(SocketStorage::new()),
            token_storage: Arc::new(
                TokenStorage::open("data/tokens").expect("failed to open token storage"),
            ),
            user_storage: Arc::new(
                UserStorage::open("data/users").expect("failed to open user storage"),
            ),
            volume_storage: Arc::new(
                VolumeStorage::open("data/volumes").expect("failed to open volume storage"),
            ),
        }
    }
}

pub async fn run() {
    let site_addr: SocketAddr = std::env::var("SITE_ADDR")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| {
            DEFAULT_SITE_ADDR
                .parse()
                .expect("DEFAULT_SITE_ADDR is invalid")
        });
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
            let default = format!("http://{}", site_addr);
            tracing::error!("REDIRECT_URI is not set or invalid, using {default}");
            url::Url::parse(&default).unwrap()
        });
    let bot_token = std::env::var("BOT_TOKEN").unwrap_or_else(|_| {
        tracing::error!("BOT_TOKEN is not set, using empty string");
        String::new()
    });

    let state = AppState::new(
        site_addr,
        client_id,
        client_secret,
        guild_id,
        redirect_uri,
        &bot_token,
    );
    let app = build_app(state);

    let listener = tokio::net::TcpListener::bind(&site_addr).await.unwrap();
    tracing::info!("listening on http://{}", &site_addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

/// Build the full application router with API routes and a static-file fallback
/// that serves the Yew SPA built by `trunk`.
pub fn build_app(state: AppState) -> Router {
    let dist_dir = std::env::var("DIST_DIR").unwrap_or_else(|_| DEFAULT_DIST_DIR.to_string());
    let index_path = format!("{dist_dir}/index.html");
    let serve_dir = ServeDir::new(&dist_dir).fallback(ServeFile::new(&index_path));

    Router::<AppState>::new()
        .nest("/health", crate::health::router())
        .nest("/login", crate::login::router())
        .nest("/logout", crate::logout::router())
        .nest("/ws", crate::ws::router())
        .fallback_service(serve_dir)
        .layer(CookieManagerLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
