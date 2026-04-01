use leptos::prelude::*;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, EnumIter)]
#[strum(serialize_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    Cupcake,
    Emerald,
    Corporate,
    Synthwave,
    Retro,
    Cyberpunk,
    Valentine,
    Halloween,
    Forest,
    Aqua,
    Lofi,
    Pastel,
    Fantasy,
    Dracula,
    Autumn,
    Business,
    Night,
    Coffee,
    Winter,
    Dim,
    Nord,
    Sunset,
}

#[component]
pub fn Header(
    theme: ReadSignal<Theme>,
    set_theme: WriteSignal<Theme>,
    set_sidebar_open: WriteSignal<bool>,
) -> impl IntoView {
    view! {
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
                <UserMenu/>
            </div>
        </header>
    }
}

#[component]
fn ThemeSwitcher(theme: ReadSignal<Theme>, set_theme: WriteSignal<Theme>) -> impl IntoView {
    view! {
        <select
            class="select select-ghost select-sm w-32"
            on:change=move |ev| {
                if let Ok(t) = event_target_value(&ev).parse::<Theme>() {
                    set_theme.set(t);
                }
            }
            prop:value=move || theme.get().to_string()
        >
            {Theme::iter().map(|t| {
                let name = t.to_string();
                view! {
                    <option value=name.clone() selected=move || theme.get() == t>{name.clone()}</option>
                }
            }).collect_view()}
        </select>
    }
}

#[cfg(feature = "hydrate")]
fn connect_ws(set_user: WriteSignal<Option<String>>) {
    use filebrowser_types::ServerMsg;
    use wasm_bindgen::prelude::*;

    let location = web_sys::window().unwrap().location();
    let protocol = location.protocol().unwrap();
    let host = location.host().unwrap();
    let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
    let url = format!("{ws_protocol}//{host}/ws");

    let ws = web_sys::WebSocket::new(&url).unwrap();
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let onmessage = Closure::<dyn FnMut(_)>::new(move |e: web_sys::MessageEvent| {
        if let Ok(buf) = e.data().dyn_into::<web_sys::js_sys::ArrayBuffer>() {
            let bytes = web_sys::js_sys::Uint8Array::new(&buf).to_vec();
            if let Ok((msg, _)) =
                bincode::decode_from_slice::<ServerMsg, _>(&bytes, bincode::config::standard())
            {
                match msg {
                    ServerMsg::Hello { username } => set_user.set(Some(username)),
                    ServerMsg::Unauthenticated => set_user.set(None),
                }
            }
        }
    });

    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();
}

#[component]
fn UserMenu() -> impl IntoView {
    let (user, set_user) = signal(None::<String>);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        connect_ws(set_user);
    });

    let user_icon = view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5.121 17.804A13.937 13.937 0 0112 16c2.5 0 4.847.655 6.879 1.804M15 10a3 3 0 11-6 0 3 3 0 016 0z"/>
        </svg>
    };

    view! {
        <div class="dropdown dropdown-end">
            <div tabindex="0" role="button" class="btn btn-ghost btn-circle">
                {user_icon}
            </div>
            <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-10 w-52 p-2 shadow">
                {move || {
                    if let Some(username) = user.get() {
                        view! {
                            <li class="menu-title"><span>{username}</span></li>
                            <li><a href=logout_href_with_redirect() rel="external" on:click=move |_| set_user.set(None)>"Logout"</a></li>
                        }.into_any()
                    } else {
                        let login_href = login_href_with_redirect();
                        view! {
                            <li><a href=login_href rel="external">"Login with Discord"</a></li>
                        }.into_any()
                    }
                }}
            </ul>
        </div>
    }
}

#[cfg(feature = "hydrate")]
fn href_with_redirect(base: &str) -> String {
    let path = web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_default();
    let encoded = web_sys::js_sys::encode_uri_component(&path);
    format!("{base}?redirect={encoded}")
}

#[cfg(not(feature = "hydrate"))]
fn href_with_redirect(base: &str) -> String {
    base.to_string()
}

fn login_href_with_redirect() -> String {
    href_with_redirect("/login")
}

fn logout_href_with_redirect() -> String {
    href_with_redirect("/logout")
}
