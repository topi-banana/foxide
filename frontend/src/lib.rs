pub mod admin;
pub mod header;
pub mod volume;
pub mod ws;

use std::rc::Rc;

use foxide_types::{ClientMsg, ServerMsg};
use wasm_bindgen::prelude::*;
use web_sys::WebSocket;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::admin::{AdminPage, AdminTokensPage, AdminVolumesPage};
use crate::header::{Header, Theme};
use crate::volume::VolumePage;
use crate::ws::{AppCtx, AppData};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/v/:id")]
    Volume { id: u64 },
    #[at("/admin")]
    Admin,
    #[at("/admin/tokens")]
    AdminTokens,
    #[at("/admin/volumes")]
    AdminVolumes,
    #[not_found]
    #[at("/404")]
    NotFound,
}

// --- App ---

pub struct App {
    data: Rc<AppData>,
    theme: Theme,
    sidebar_open: bool,
    ws: Option<WebSocket>,
    /// Closures kept alive for the lifetime of the WebSocket.
    _onmessage: Option<Closure<dyn FnMut(web_sys::MessageEvent)>>,
}

pub enum AppMsg {
    WsRecv(ServerMsg),
    Send(ClientMsg),
    SetTheme(Theme),
    SetSidebarOpen(bool),
}

impl App {
    fn build_ctx(&self, ctx: &Context<Self>) -> AppCtx {
        AppCtx {
            data: self.data.clone(),
            theme: self.theme,
            sidebar_open: self.sidebar_open,
            send: ctx.link().callback(AppMsg::Send),
            set_theme: ctx.link().callback(AppMsg::SetTheme),
            set_sidebar_open: ctx.link().callback(AppMsg::SetSidebarOpen),
        }
    }

    fn data_mut(&mut self) -> &mut AppData {
        Rc::make_mut(&mut self.data)
    }

    fn handle_server_msg(&mut self, msg: ServerMsg) -> Vec<ClientMsg> {
        let mut follow_ups = Vec::new();
        let data = self.data_mut();
        match msg {
            ServerMsg::Hello {
                username,
                avatar_url,
            } => {
                data.username = Some(username);
                data.avatar_url = avatar_url;
                data.ready = true;
                follow_ups.push(ClientMsg::GetMyVolumes);
            }
            ServerMsg::Unauthenticated => {
                data.username = None;
                data.avatar_url = None;
                data.ready = true;
            }
            ServerMsg::MyVolumes { volumes } => {
                data.volumes = volumes.into_iter().map(|v| (v.id, v.name)).collect();
            }
            ServerMsg::Admin(resp) => {
                data.admin.apply(resp);
            }
            ServerMsg::Browse(resp) => {
                data.browse.apply(resp);
            }
        }
        follow_ups
    }

    fn open_ws(&mut self, ctx: &Context<Self>) {
        let location = web_sys::window().unwrap().location();
        let protocol = location.protocol().unwrap_or_else(|_| "http:".into());
        let host = location.host().unwrap_or_default();
        let scheme = if protocol == "https:" { "wss:" } else { "ws:" };
        let url = format!("{scheme}//{host}/ws");

        let ws = match WebSocket::new(&url) {
            Ok(ws) => ws,
            Err(e) => {
                tracing_console_error("failed to open WebSocket", &e);
                return;
            }
        };
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let link = ctx.link().clone();
        let onmessage = Closure::<dyn FnMut(_)>::new(move |e: web_sys::MessageEvent| {
            let Ok(buf) = e.data().dyn_into::<js_sys::ArrayBuffer>() else {
                return;
            };
            let bytes = js_sys::Uint8Array::new(&buf).to_vec();
            match rmp_serde::from_slice::<ServerMsg>(&bytes) {
                Ok(msg) => link.send_message(AppMsg::WsRecv(msg)),
                Err(e) => tracing_console_error(
                    "failed to decode ServerMsg",
                    &JsValue::from_str(&e.to_string()),
                ),
            }
        });
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

        self.ws = Some(ws);
        self._onmessage = Some(onmessage);
    }

    fn send_client_msg(&self, msg: &ClientMsg) {
        let Some(ws) = self.ws.as_ref() else { return };
        match rmp_serde::to_vec(msg) {
            Ok(bytes) => {
                if let Err(e) = ws.send_with_u8_array(&bytes) {
                    tracing_console_error("failed to send ClientMsg", &e);
                }
            }
            Err(e) => tracing_console_error(
                "failed to encode ClientMsg",
                &JsValue::from_str(&e.to_string()),
            ),
        }
    }
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let theme = read_theme_cookie().unwrap_or_default();
        let sidebar_open = read_sidebar_cookie().unwrap_or(false);

        Self {
            data: Rc::new(AppData::default()),
            theme,
            sidebar_open,
            ws: None,
            _onmessage: None,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.open_ws(ctx);
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::WsRecv(server_msg) => {
                let follow_ups = self.handle_server_msg(server_msg);
                for m in follow_ups {
                    self.send_client_msg(&m);
                }
                true
            }
            AppMsg::Send(client_msg) => {
                self.send_client_msg(&client_msg);
                false
            }
            AppMsg::SetTheme(t) => {
                if self.theme != t {
                    self.theme = t;
                    write_theme_cookie(t);
                    apply_theme_attribute(t);
                    true
                } else {
                    false
                }
            }
            AppMsg::SetSidebarOpen(open) => {
                if self.sidebar_open != open {
                    self.sidebar_open = open;
                    write_sidebar_cookie(open);
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // Reflect theme on the <html> element for daisyUI.
        apply_theme_attribute(self.theme);

        let app_ctx = self.build_ctx(ctx);

        let sidebar_class = if self.sidebar_open {
            "bg-base-200 transition-all duration-300 overflow-hidden flex-shrink-0 w-64"
        } else {
            "bg-base-200 transition-all duration-300 overflow-hidden flex-shrink-0 w-0"
        };

        let sidebar_volumes: Html = if self.data.ready {
            let items: Html = self
                .data
                .volumes
                .iter()
                .map(|(id, name)| {
                    let href = format!("/v/{id}");
                    html! { <li><a href={href}>{name}</a></li> }
                })
                .collect();
            html! { <ul class="menu">{items}</ul> }
        } else {
            html! {
                <div class="flex justify-center py-4">
                    <span class="loading loading-spinner loading-sm"></span>
                </div>
            }
        };

        html! {
            <ContextProvider<AppCtx> context={app_ctx}>
                <BrowserRouter>
                    <div class="flex h-screen">
                        <aside class={sidebar_class}>
                            <nav class="w-64 h-full p-4">
                                <ul class="menu">
                                    <li><a href="/">{"Home"}</a></li>
                                </ul>
                                {sidebar_volumes}
                                <ul class="menu">
                                    <li><a href="/admin">{"Admin"}</a></li>
                                </ul>
                            </nav>
                        </aside>

                        <div class="flex flex-col flex-1 min-w-0">
                            <Header />
                            <main class="flex-1 overflow-auto p-6">
                                <Switch<Route> render={switch_routes} />
                            </main>
                        </div>
                    </div>
                </BrowserRouter>
            </ContextProvider<AppCtx>>
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        if let Some(ws) = self.ws.take() {
            let _ = ws.close();
        }
    }
}

fn switch_routes(route: Route) -> Html {
    match route {
        Route::Home => html! { <HomePage /> },
        Route::Volume { id } => html! { <VolumePage {id} /> },
        Route::Admin => html! { <AdminPage /> },
        Route::AdminTokens => html! { <AdminTokensPage /> },
        Route::AdminVolumes => html! { <AdminVolumesPage /> },
        Route::NotFound => html! {
            <div class="text-center py-8">
                <h1 class="text-2xl font-bold">{"Not found."}</h1>
            </div>
        },
    }
}

// --- HomePage ---

pub struct HomePage;

impl Component for HomePage {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <h1 class="text-2xl font-bold mb-4">{"Welcome to Foxide"}</h1>
                <p class="text-base-content/70">{"Select a folder from the sidebar to get started."}</p>
            </>
        }
    }
}

// --- Cookie helpers ---

pub(crate) fn html_document() -> Option<web_sys::HtmlDocument> {
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

fn read_sidebar_cookie() -> Option<bool> {
    let cookies = html_document()?.cookie().ok()?;
    cookies
        .split(';')
        .filter_map(|c| c.trim().strip_prefix("sidebar="))
        .next()?
        .parse()
        .ok()
}

fn write_sidebar_cookie(open: bool) {
    if let Some(doc) = html_document() {
        let _ = doc.set_cookie(&format!(
            "sidebar={open};path=/;max-age=31536000;SameSite=Lax"
        ));
    }
}

fn apply_theme_attribute(theme: Theme) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document())
        && let Some(html_el) = doc.document_element()
    {
        let _ = html_el.set_attribute("data-theme", &theme.to_string());
    }
}

fn tracing_console_error(message: &str, err: &JsValue) {
    web_sys::console::error_2(&JsValue::from_str(message), err);
}
