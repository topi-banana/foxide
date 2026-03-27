mod health;
mod login;

use axum::Router;
use axum::extract::FromRef;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::trace::TraceLayer;

use filebrowser_frontend::{App, shell};

/// Application-wide shared state.
#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
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
    let state = AppState { leptos_options };
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

    Router::new()
        .nest("/health", crate::health::router::<AppState>())
        .nest("/login", crate::login::router::<AppState>())
        .leptos_routes(&state, routes, {
            let options = state.leptos_options.clone();
            move || shell(options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
