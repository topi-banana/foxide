use leptos::prelude::*;

#[cfg(feature = "hydrate")]
fn connect_tokens_ws(
    set_tokens: WriteSignal<Vec<(u64, String, String)>>,
    set_error: WriteSignal<Option<String>>,
    set_unauthorized: WriteSignal<bool>,
) {
    use filebrowser_types::{AdminAction, AdminResponse, ClientMsg, ServerMsg};
    use wasm_bindgen::prelude::*;

    let location = web_sys::window().unwrap().location();
    let protocol = location.protocol().unwrap();
    let host = location.host().unwrap();
    let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
    let url = format!("{ws_protocol}//{host}/ws");

    let ws = web_sys::WebSocket::new(&url).unwrap();
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let ws_clone = ws.clone();
    let onopen = Closure::<dyn FnMut()>::new(move || {
        let msg = ClientMsg::Admin(AdminAction::GetTokens);
        let bytes = wincode::serialize(&msg).unwrap();
        let _ = ws_clone.send_with_u8_array(&bytes);
    });
    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    onopen.forget();

    let onmessage = Closure::<dyn FnMut(_)>::new(move |e: web_sys::MessageEvent| {
        if let Ok(buf) = e.data().dyn_into::<web_sys::js_sys::ArrayBuffer>() {
            let bytes = web_sys::js_sys::Uint8Array::new(&buf).to_vec();
            if let Ok(msg) = wincode::deserialize::<ServerMsg>(&bytes) {
                match msg {
                    ServerMsg::Admin(admin_resp) => match admin_resp {
                        AdminResponse::Tokens { tokens } => {
                            set_tokens.set(
                                tokens
                                    .into_iter()
                                    .map(|t| (t.user_id, t.username, t.expires))
                                    .collect(),
                            );
                            set_error.set(None);
                            set_unauthorized.set(false);
                        }
                        AdminResponse::Error { message } => {
                            set_error.set(Some(message));
                        }
                        AdminResponse::Unauthorized => {
                            set_unauthorized.set(true);
                        }
                        _ => {}
                    },
                    ServerMsg::Unauthenticated => {
                        set_unauthorized.set(true);
                    }
                    _ => {}
                }
            }
        }
    });
    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();
}

#[component]
pub fn AdminTokensPage() -> impl IntoView {
    let (tokens, set_tokens) = signal(Vec::<(u64, String, String)>::new());
    let (error, set_error) = signal(None::<String>);
    let (unauthorized, set_unauthorized) = signal(false);

    #[cfg(not(feature = "hydrate"))]
    let _ = (set_tokens, set_error, set_unauthorized);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        connect_tokens_ws(set_tokens, set_error, set_unauthorized);
    });

    view! {
        <div class="max-w-4xl mx-auto">
            <h1 class="text-2xl font-bold mb-6">"Active Sessions"</h1>

            {move || unauthorized.get().then(|| view! {
                <div class="alert alert-error mb-4">
                    <span>"You are not authorized to access this page."</span>
                </div>
            })}

            {move || error.get().map(|msg| view! {
                <div class="alert alert-warning mb-4">
                    <span>{msg}</span>
                </div>
            })}

            <div class="overflow-x-auto">
                <table class="table table-zebra w-full">
                    <thead>
                        <tr>
                            <th>"User ID"</th>
                            <th>"Username"</th>
                            <th>"Expires"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || tokens.get().into_iter().map(|(user_id, username, expires)| {
                            view! {
                                <tr>
                                    <td>{user_id}</td>
                                    <td>{username}</td>
                                    <td>{expires}</td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>

            {move || tokens.get().is_empty().then(|| view! {
                <p class="text-base-content/70 mt-4">"No active sessions."</p>
            })}
        </div>
    }
}
