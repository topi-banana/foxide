use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;

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
    let (theme, set_theme) = signal("light".to_string());

    view! {
        <Title text="Filebrowser"/>
        <Html attr:data-theme=theme/>
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
                // Header
                <header class="navbar bg-base-300 flex-shrink-0">
                    <div class="navbar-start">
                        <button
                            class="btn btn-ghost btn-square"
                            on:click=move |_| set_sidebar_open.update(|v| *v = !*v)
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
                            </svg>
                        </button>
                    </div>
                    <div class="navbar-center">
                        <span class="text-lg font-bold">"Filebrowser"</span>
                    </div>
                    <div class="navbar-end gap-1">
                        <ThemeSwitcher theme set_theme/>
                        <button class="btn btn-ghost btn-circle avatar">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5.121 17.804A13.937 13.937 0 0112 16c2.5 0 4.847.655 6.879 1.804M15 10a3 3 0 11-6 0 3 3 0 016 0z"/>
                            </svg>
                        </button>
                    </div>
                </header>

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

const THEMES: &[&str] = &[
    "light",
    "dark",
    "cupcake",
    "emerald",
    "corporate",
    "synthwave",
    "retro",
    "cyberpunk",
    "valentine",
    "halloween",
    "forest",
    "aqua",
    "lofi",
    "pastel",
    "fantasy",
    "dracula",
    "autumn",
    "business",
    "night",
    "coffee",
    "winter",
    "dim",
    "nord",
    "sunset",
];

#[component]
fn ThemeSwitcher(theme: ReadSignal<String>, set_theme: WriteSignal<String>) -> impl IntoView {
    view! {
        <select
            class="select select-ghost select-sm w-32"
            on:change=move |ev| {
                set_theme.set(event_target_value(&ev));
            }
            prop:value=theme
        >
            {THEMES.iter().map(|t| view! {
                <option value={*t} selected=move || theme.get() == *t>{*t}</option>
            }).collect_view()}
        </select>
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
