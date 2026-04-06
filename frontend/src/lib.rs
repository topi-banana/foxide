mod admin;
mod header;
mod volume;
pub mod ws;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;

use admin::{AdminPage, AdminTokensPage, AdminVolumesPage};
use header::{Header, Theme};
use volume::VolumePage;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <Stylesheet id="leptos" href="/pkg/filebrowser.css"/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let ws = ws::WsCtx::new();
    provide_context(ws);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        ws.connect();
    });

    let (sidebar_open, set_sidebar_open) = signal(false);
    let (theme, set_theme) = signal(Theme::Light);

    Effect::new(move |prev: Option<()>| {
        if prev.is_none() {
            if let Some(t) = read_theme_cookie() {
                set_theme.set(t);
            }
            if let Some(s) = read_sidebar_cookie() {
                set_sidebar_open.set(s);
            }
        }
        write_theme_cookie(theme.get());
        write_sidebar_cookie(sidebar_open.get());
    });

    view! {
        <Title text="Filebrowser"/>
        <Html attr:data-theme=move || theme.get().to_string()/>
        <div class="flex h-screen">
            // Sidebar
            <aside
                class="bg-base-200 transition-all duration-300 overflow-hidden flex-shrink-0"
                class:w-64={sidebar_open}
                class:w-0={move || !sidebar_open.get()}
            >
                <nav class="w-64 h-full p-4">
                    <ul class="menu">
                        <li><a href="/">"Home"</a></li>
                    </ul>
                    {move || {
                        if !ws.ready.get() {
                            view! {
                                <div class="flex justify-center py-4">
                                    <span class="loading loading-spinner loading-sm"></span>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <ul class="menu">
                                    {move || ws.volumes.get().into_iter().map(|(id, name)| {
                                        let href = format!("/v/{id}");
                                        view! {
                                            <li><a href=href>{name}</a></li>
                                        }
                                    }).collect_view()}
                                </ul>
                            }.into_any()
                        }
                    }}
                    <ul class="menu">
                        <li><a href="/admin">"Admin"</a></li>
                    </ul>
                </nav>
            </aside>

            // Main area
            <div class="flex flex-col flex-1 min-w-0">
                <Header theme set_theme set_sidebar_open/>

                // Content
                <main class="flex-1 overflow-auto p-6">
                    <Router>
                        <Routes fallback=|| "Not found.">
                            <Route path=path!("/") view=HomePage/>
                            <Route path=path!("/v/:id") view=VolumePage/>
                            <Route path=path!("/admin") view=AdminPage/>
                            <Route path=path!("/admin/tokens") view=AdminTokensPage/>
                            <Route path=path!("/admin/volumes") view=AdminVolumesPage/>
                        </Routes>
                    </Router>
                </main>
            </div>
        </div>
    }
}

#[cfg(feature = "hydrate")]
fn html_document() -> Option<web_sys::HtmlDocument> {
    use wasm_bindgen::JsCast;
    web_sys::window()?
        .document()?
        .dyn_into::<web_sys::HtmlDocument>()
        .ok()
}

#[cfg(feature = "hydrate")]
fn read_theme_cookie() -> Option<Theme> {
    let cookies = html_document()?.cookie().ok()?;
    cookies
        .split(';')
        .filter_map(|c| c.trim().strip_prefix("theme="))
        .next()?
        .parse()
        .ok()
}

#[cfg(not(feature = "hydrate"))]
fn read_theme_cookie() -> Option<Theme> {
    None
}

#[cfg(feature = "hydrate")]
fn write_theme_cookie(theme: Theme) {
    if let Some(doc) = html_document() {
        let _ = doc.set_cookie(&format!(
            "theme={theme};path=/;max-age=31536000;SameSite=Lax"
        ));
    }
}

#[cfg(not(feature = "hydrate"))]
fn write_theme_cookie(_theme: Theme) {}

#[cfg(feature = "hydrate")]
fn read_sidebar_cookie() -> Option<bool> {
    let cookies = html_document()?.cookie().ok()?;
    cookies
        .split(';')
        .filter_map(|c| c.trim().strip_prefix("sidebar="))
        .next()?
        .parse()
        .ok()
}

#[cfg(not(feature = "hydrate"))]
fn read_sidebar_cookie() -> Option<bool> {
    None
}

#[cfg(feature = "hydrate")]
fn write_sidebar_cookie(open: bool) {
    if let Some(doc) = html_document() {
        let _ = doc.set_cookie(&format!(
            "sidebar={open};path=/;max-age=31536000;SameSite=Lax"
        ));
    }
}

#[cfg(not(feature = "hydrate"))]
fn write_sidebar_cookie(_open: bool) {}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <h1 class="text-2xl font-bold mb-4">"Welcome to Filebrowser"</h1>
        <p class="text-base-content/70">"Select a folder from the sidebar to get started."</p>
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
