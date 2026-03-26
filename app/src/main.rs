#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use tracing::Level;
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".parse().unwrap()),
        )
        .init();

    filebrowser_backend::run().await;
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // This binary is only intended to be built with the `ssr` feature.
    // WASM hydration is handled by the frontend crate.
}
