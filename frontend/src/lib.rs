mod header;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;

use header::{Header, Theme};

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
    let (sidebar_open, set_sidebar_open) = signal(false);
    let initial_theme = read_theme_cookie().unwrap_or(Theme::Light);
    let (theme, set_theme) = signal(initial_theme);

    Effect::new(move |_| {
        write_theme_cookie(theme.get());
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
                        <li><a class="active">"Home"</a></li>
                        <li><a>"Documents"</a></li>
                        <li><a>"Photos"</a></li>
                        <li><a>"Settings"</a></li>
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
                        </Routes>
                    </Router>
                </main>
            </div>
        </div>
    }
}

fn html_document() -> Option<web_sys::HtmlDocument> {
    use wasm_bindgen::JsCast;
    web_sys::window()?
        .document()?
        .dyn_into::<web_sys::HtmlDocument>()
        .ok()
}

fn read_theme_cookie() -> Option<Theme> {
    let cookies = html_document()?.cookie().ok()?;
    cookies
        .split(';')
        .filter_map(|c| c.trim().strip_prefix("theme="))
        .next()?
        .parse()
        .ok()
}

fn write_theme_cookie(theme: Theme) {
    if let Some(doc) = html_document() {
        let _ = doc.set_cookie(&format!(
            "theme={theme};path=/;max-age=31536000;SameSite=Lax"
        ));
    }
}

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
