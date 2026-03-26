#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    filebrowser_app::server::run().await;
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // This binary is only intended to be built with the `ssr` feature.
    // WASM hydration is handled by the frontend crate.
}
