#[cfg(feature = "hydrate")]
use filebrowser_types::AdminResponse;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use std::sync::Arc;

/// Centralised WebSocket context shared across all pages.
///
/// Created once in [`App`], provided via Leptos context.
/// Pages register an admin callback with [`set_on_admin`] and send
/// actions with [`send`].
#[derive(Clone, Copy)]
pub struct WsCtx {
    /// Logged-in username.  `None` until `Hello` arrives (or if unauthenticated).
    pub username: RwSignal<Option<String>>,
    /// Discord avatar URL.  `None` if unauthenticated or no avatar set.
    pub avatar_url: RwSignal<Option<String>>,
    /// `true` once the server has replied with `Hello` or `Unauthenticated`.
    pub ready: RwSignal<bool>,
    /// Volumes accessible to the current user (populated after Hello).
    pub volumes: RwSignal<Vec<(u64, String)>>,

    #[cfg(feature = "hydrate")]
    ws: StoredValue<Option<web_sys::WebSocket>>,
    #[cfg(feature = "hydrate")]
    #[allow(clippy::type_complexity)]
    on_admin: StoredValue<Option<Arc<dyn Fn(AdminResponse) + Send + Sync>>>,
}

impl Default for WsCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl WsCtx {
    pub fn new() -> Self {
        Self {
            username: RwSignal::new(None),
            avatar_url: RwSignal::new(None),
            ready: RwSignal::new(false),
            volumes: RwSignal::new(vec![]),
            #[cfg(feature = "hydrate")]
            ws: StoredValue::new(None),
            #[cfg(feature = "hydrate")]
            on_admin: StoredValue::new(None),
        }
    }

    /// Open the WebSocket connection.  Call once from [`App`].
    #[cfg(feature = "hydrate")]
    pub fn connect(&self) {
        use filebrowser_types::ServerMsg;
        use wasm_bindgen::prelude::*;

        let location = web_sys::window().unwrap().location();
        let protocol = location.protocol().unwrap();
        let host = location.host().unwrap();
        let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
        let url = format!("{ws_protocol}//{host}/ws");

        let ws = web_sys::WebSocket::new(&url).unwrap();
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let ctx = *self;
        let ws_ref = ws.clone();
        let onmessage = Closure::<dyn FnMut(_)>::new(move |e: web_sys::MessageEvent| {
            if let Ok(buf) = e.data().dyn_into::<web_sys::js_sys::ArrayBuffer>() {
                let bytes = web_sys::js_sys::Uint8Array::new(&buf).to_vec();
                if let Ok(msg) = wincode::deserialize::<ServerMsg>(&bytes) {
                    match msg {
                        ServerMsg::Hello {
                            username,
                            avatar_url,
                        } => {
                            ctx.username.set(Some(username));
                            ctx.avatar_url.set(avatar_url);
                            ctx.ready.set(true);
                            // Fetch accessible volumes for the sidebar
                            let req = filebrowser_types::ClientMsg::GetMyVolumes;
                            let bytes = wincode::serialize(&req).unwrap();
                            let _ = ws_ref.send_with_u8_array(&bytes);
                        }
                        ServerMsg::Unauthenticated => {
                            ctx.username.set(None);
                            ctx.avatar_url.set(None);
                            ctx.ready.set(true);
                        }
                        ServerMsg::Admin(resp) => {
                            ctx.on_admin.with_value(|handler| {
                                if let Some(handler) = handler {
                                    handler(resp);
                                }
                            });
                        }
                        ServerMsg::MyVolumes { volumes } => {
                            ctx.volumes
                                .set(volumes.into_iter().map(|v| (v.id, v.name)).collect());
                        }
                    }
                }
            }
        });
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        self.ws.set_value(Some(ws));
    }

    /// Send an admin action over the shared connection.
    #[cfg(feature = "hydrate")]
    pub fn send(&self, action: filebrowser_types::AdminAction) {
        use filebrowser_types::ClientMsg;
        self.ws.with_value(|ws| {
            if let Some(ws) = ws {
                let msg = ClientMsg::Admin(action);
                let bytes = wincode::serialize(&msg).unwrap();
                let _ = ws.send_with_u8_array(&bytes);
            }
        });
    }

    /// Register the callback that receives [`AdminResponse`] messages.
    ///
    /// Only one handler is active at a time — each page overwrites the
    /// previous one on mount.
    #[cfg(feature = "hydrate")]
    pub fn set_on_admin(&self, handler: impl Fn(AdminResponse) + Send + Sync + 'static) {
        self.on_admin.set_value(Some(Arc::new(handler)));
    }
}
