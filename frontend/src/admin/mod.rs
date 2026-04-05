mod tokens;
mod volumes;

pub use tokens::AdminTokensPage;
pub use volumes::AdminVolumesPage;

use leptos::prelude::*;

#[cfg(feature = "hydrate")]
fn connect_admin_ws(
    set_roles: WriteSignal<Vec<(u64, String)>>,
    set_admin_role_id: WriteSignal<Option<u64>>,
    set_error: WriteSignal<Option<String>>,
    set_unauthorized: WriteSignal<bool>,
    ws_handle: StoredValue<Option<web_sys::WebSocket>>,
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

    // On open, send GetRoles request
    let ws_clone = ws.clone();
    let onopen = Closure::<dyn FnMut()>::new(move || {
        let msg = ClientMsg::Admin(AdminAction::GetRoles);
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
                        AdminResponse::Roles {
                            roles,
                            admin_role_id,
                        } => {
                            set_roles.set(roles.into_iter().map(|r| (r.id, r.name)).collect());
                            set_admin_role_id.set(admin_role_id);
                            set_error.set(None);
                            set_unauthorized.set(false);
                        }
                        AdminResponse::AdminRoleUpdated { role_id } => {
                            set_admin_role_id.set(Some(role_id));
                            set_error.set(None);
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

    ws_handle.set_value(Some(ws));
}

#[cfg(feature = "hydrate")]
fn send_action(
    ws_handle: StoredValue<Option<web_sys::WebSocket>>,
    action: filebrowser_types::AdminAction,
) {
    use filebrowser_types::ClientMsg;

    ws_handle.with_value(|ws| {
        if let Some(ws) = ws {
            let msg = ClientMsg::Admin(action);
            let bytes = wincode::serialize(&msg).unwrap();
            let _ = ws.send_with_u8_array(&bytes);
        }
    });
}

#[component]
pub fn AdminPage() -> impl IntoView {
    let (roles, set_roles) = signal(Vec::<(u64, String)>::new());
    let (admin_role_id, set_admin_role_id) = signal(None::<u64>);
    let (error, set_error) = signal(None::<String>);
    let (unauthorized, set_unauthorized) = signal(false);

    #[cfg(not(feature = "hydrate"))]
    let _ = (set_roles, set_admin_role_id, set_error, set_unauthorized);

    #[cfg(feature = "hydrate")]
    let ws_handle = StoredValue::new(None::<web_sys::WebSocket>);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        connect_admin_ws(
            set_roles,
            set_admin_role_id,
            set_error,
            set_unauthorized,
            ws_handle,
        );
    });

    view! {
        <div class="max-w-2xl mx-auto">
            <h1 class="text-2xl font-bold mb-6">"Admin Settings"</h1>

            {move || unauthorized.get().then(|| view! {
                <div class="alert alert-error mb-4">
                    <span>"You are not authorized to access admin settings."</span>
                </div>
            })}

            {move || error.get().map(|msg| view! {
                <div class="alert alert-warning mb-4">
                    <span>{msg}</span>
                </div>
            })}

            <div class="card bg-base-200 shadow-xl">
                <div class="card-body">
                    <h2 class="card-title">"Admin Role"</h2>
                    <p class="text-base-content/70">"Select which Discord role grants admin access."</p>

                    <div class="form-control w-full mt-4">
                        <select
                            class="select select-bordered w-full"
                            prop:value=move || admin_role_id.get().map(|id| id.to_string()).unwrap_or_default()
                            on:change=move |ev| {
                                let _value = event_target_value(&ev);
                                #[cfg(feature = "hydrate")]
                                if let Ok(role_id) = _value.parse::<u64>() {
                                    send_action(ws_handle, filebrowser_types::AdminAction::SetAdminRole { role_id });
                                }
                            }
                            disabled=move || unauthorized.get() || roles.get().is_empty()
                        >
                            <option value="" disabled selected=move || admin_role_id.get().is_none()>
                                "Select a role..."
                            </option>
                            {move || roles.get().into_iter().map(|(id, name)| {
                                let id_str = id.to_string();
                                let selected = move || admin_role_id.get() == Some(id);
                                view! {
                                    <option value=id_str.clone() selected=selected>{name.clone()}</option>
                                }
                            }).collect_view()}
                        </select>
                    </div>
                </div>
            </div>

            <div class="card bg-base-200 shadow-xl mt-6">
                <div class="card-body">
                    <h2 class="card-title">"Management"</h2>
                    <ul class="menu">
                        <li><a href="/admin/tokens">"Active Sessions"</a></li>
                        <li><a href="/admin/volumes">"Volumes"</a></li>
                    </ul>
                </div>
            </div>
        </div>
    }
}
