#[cfg(feature = "ssr")]
pub mod server {
    use axum::Router;
    use axum::routing::get;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};

    use filebrowser_backend::api;
    use filebrowser_frontend::{App, shell};

    /// API-only routes. State-generic so they work both standalone and merged.
    pub fn api_routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
        Router::new().route("/api/health", get(api::health))
    }

    /// Build the full application router with API routes and Leptos SSR.
    pub fn build_app(leptos_options: LeptosOptions) -> Router {
        let routes = generate_route_list(App);

        Router::new()
            .merge(api_routes::<LeptosOptions>())
            .leptos_routes(&leptos_options, routes, {
                let options = leptos_options.clone();
                move || shell(options.clone())
            })
            .fallback(leptos_axum::file_and_error_handler(shell))
            .with_state(leptos_options)
    }

    pub async fn run() {
        let conf = get_configuration(None).unwrap();
        let leptos_options = conf.leptos_options;
        let addr = leptos_options.site_addr;
        let app = build_app(leptos_options);

        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        println!("listening on http://{}", &addr);
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    }
}
