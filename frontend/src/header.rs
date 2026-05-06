use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
use wasm_bindgen::JsCast;
use web_sys::HtmlSelectElement;
use yew::prelude::*;

use crate::ws::AppCtx;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Display, EnumString, EnumIter)]
#[strum(serialize_all = "lowercase")]
pub enum Theme {
    #[default]
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

// --- Header ---

pub struct Header {
    ctx: AppCtx,
    _ctx_handle: ContextHandle<AppCtx>,
}

pub enum HeaderMsg {
    ContextChanged(AppCtx),
    ToggleSidebar,
}

impl Component for Header {
    type Message = HeaderMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (app_ctx, handle) = ctx
            .link()
            .context::<AppCtx>(ctx.link().callback(HeaderMsg::ContextChanged))
            .expect("AppCtx not provided");
        Self {
            ctx: app_ctx,
            _ctx_handle: handle,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            HeaderMsg::ContextChanged(c) => {
                self.ctx = c;
                true
            }
            HeaderMsg::ToggleSidebar => {
                self.ctx.set_sidebar_open.emit(!self.ctx.sidebar_open);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let toggle = ctx.link().callback(|_| HeaderMsg::ToggleSidebar);
        html! {
            <header class="navbar bg-base-300 flex-shrink-0">
                <div class="navbar-start">
                    <button class="btn btn-ghost btn-square" onclick={toggle}>
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
                        </svg>
                    </button>
                </div>
                <div class="navbar-center">
                    <span class="text-lg font-bold">{"Filebrowser"}</span>
                </div>
                <div class="navbar-end gap-1">
                    <ThemeSwitcher />
                    <UserMenu />
                </div>
            </header>
        }
    }
}

// --- ThemeSwitcher ---

pub struct ThemeSwitcher {
    ctx: AppCtx,
    _ctx_handle: ContextHandle<AppCtx>,
}

pub enum ThemeMsg {
    ContextChanged(AppCtx),
    Change(Theme),
}

impl Component for ThemeSwitcher {
    type Message = ThemeMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (app_ctx, handle) = ctx
            .link()
            .context::<AppCtx>(ctx.link().callback(ThemeMsg::ContextChanged))
            .expect("AppCtx not provided");
        Self {
            ctx: app_ctx,
            _ctx_handle: handle,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ThemeMsg::ContextChanged(c) => {
                self.ctx = c;
                true
            }
            ThemeMsg::Change(t) => {
                self.ctx.set_theme.emit(t);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let theme = self.ctx.theme;
        let onchange = ctx.link().callback(|ev: Event| {
            let target: HtmlSelectElement = ev.target().unwrap().dyn_into().unwrap();
            let parsed = target.value().parse::<Theme>().unwrap_or_default();
            ThemeMsg::Change(parsed)
        });
        html! {
            <select class="select select-ghost select-sm w-32" {onchange} value={theme.to_string()}>
                { for Theme::iter().map(|t| {
                    let selected = t == theme;
                    let name = t.to_string();
                    html! { <option value={name.clone()} {selected}>{name}</option> }
                }) }
            </select>
        }
    }
}

// --- UserMenu ---

pub struct UserMenu {
    ctx: AppCtx,
    _ctx_handle: ContextHandle<AppCtx>,
}

pub enum UserMenuMsg {
    ContextChanged(AppCtx),
}

impl Component for UserMenu {
    type Message = UserMenuMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (app_ctx, handle) = ctx
            .link()
            .context::<AppCtx>(ctx.link().callback(UserMenuMsg::ContextChanged))
            .expect("AppCtx not provided");
        Self {
            ctx: app_ctx,
            _ctx_handle: handle,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            UserMenuMsg::ContextChanged(c) => {
                self.ctx = c;
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let avatar = match &self.ctx.data.avatar_url {
            Some(url) => html! {
                <div class="w-8 rounded-full">
                    <img src={url.to_string()} alt="avatar"/>
                </div>
            },
            None => html! {
                <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5.121 17.804A13.937 13.937 0 0112 16c2.5 0 4.847.655 6.879 1.804M15 10a3 3 0 11-6 0 3 3 0 016 0z"/>
                </svg>
            },
        };

        let menu = match &self.ctx.data.username {
            Some(username) => html! {
                <>
                    <li class="menu-title"><span>{username}</span></li>
                    <li><a href={logout_href_with_redirect()} rel="external">{"Logout"}</a></li>
                </>
            },
            None => {
                let login_href = login_href_with_redirect();
                html! {
                    <li><a href={login_href} rel="external">{"Login with Discord"}</a></li>
                }
            }
        };

        html! {
            <div class="dropdown dropdown-end">
                <div tabindex="0" role="button" class="btn btn-ghost btn-circle avatar">{avatar}</div>
                <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-10 w-52 p-2 shadow">
                    {menu}
                </ul>
            </div>
        }
    }
}

fn href_with_redirect(base: &str) -> String {
    let path = web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_default();
    let encoded = js_sys::encode_uri_component(&path);
    format!("{base}?redirect={encoded}")
}

fn login_href_with_redirect() -> String {
    href_with_redirect("/login")
}

fn logout_href_with_redirect() -> String {
    href_with_redirect("/logout")
}
