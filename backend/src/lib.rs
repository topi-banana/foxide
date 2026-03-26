mod health;
mod login;

use axum::Router;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::trace::TraceLayer;

use filebrowser_frontend::{App, shell};

pub async fn run() {
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let app = build_app(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

/// Build the full application router with API routes and Leptos SSR.
pub fn build_app(leptos_options: LeptosOptions) -> Router {
    let routes = generate_route_list(App);

    Router::new()
        .nest("/health", crate::health::router::<LeptosOptions>())
        .nest("/login", crate::login::router::<LeptosOptions>())
        .leptos_routes(&leptos_options, routes, {
            let options = leptos_options.clone();
            move || shell(options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(TraceLayer::new_for_http())
        .with_state(leptos_options)
}
