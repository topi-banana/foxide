use leptos::prelude::*;

#[cfg(feature = "hydrate")]
fn connect_volumes_ws(
    set_volumes: WriteSignal<Vec<(u64, String, String, u64)>>,
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

    let ws_clone = ws.clone();
    let onopen = Closure::<dyn FnMut()>::new(move || {
        let msg = ClientMsg::Admin(AdminAction::GetVolumes);
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
                        AdminResponse::Volumes { volumes } => {
                            set_volumes.set(
                                volumes
                                    .into_iter()
                                    .map(|v| (v.id, v.name, v.path, v.role_id))
                                    .collect(),
                            );
                            set_error.set(None);
                            set_unauthorized.set(false);
                        }
                        AdminResponse::VolumeAdded { volume } => {
                            set_volumes.update(|vols| {
                                vols.push((volume.id, volume.name, volume.path, volume.role_id));
                            });
                            set_error.set(None);
                        }
                        AdminResponse::VolumeRemoved { id } => {
                            set_volumes.update(|vols| {
                                vols.retain(|(vid, _, _, _)| *vid != id);
                            });
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
pub fn AdminVolumesPage() -> impl IntoView {
    let (volumes, set_volumes) = signal(Vec::<(u64, String, String, u64)>::new());
    let (error, set_error) = signal(None::<String>);
    let (unauthorized, set_unauthorized) = signal(false);

    // Form fields
    let (name, set_name) = signal(String::new());
    let (path, set_path) = signal(String::new());
    let (role_id, set_role_id) = signal(String::new());

    #[cfg(not(feature = "hydrate"))]
    let _ = (
        set_volumes,
        set_error,
        set_unauthorized,
        set_name,
        set_path,
        set_role_id,
    );

    #[cfg(feature = "hydrate")]
    let ws_handle = StoredValue::new(None::<web_sys::WebSocket>);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        connect_volumes_ws(set_volumes, set_error, set_unauthorized, ws_handle);
    });

    let add_disabled = move || {
        unauthorized.get()
            || name.get().is_empty()
            || path.get().is_empty()
            || role_id.get().parse::<u64>().is_err()
    };

    view! {
        <div class="max-w-4xl mx-auto">
            <h1 class="text-2xl font-bold mb-6">"Volumes"</h1>

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

            // Add volume form
            <div class="card bg-base-200 shadow-xl mb-6">
                <div class="card-body">
                    <h2 class="card-title">"Add Volume"</h2>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mt-2">
                        <div class="form-control">
                            <label class="label"><span class="label-text">"Name"</span></label>
                            <input
                                type="text"
                                class="input input-bordered w-full"
                                placeholder="My Files"
                                prop:value=move || name.get()
                                on:input=move |ev| {
                                    let _v = event_target_value(&ev);
                                    #[cfg(feature = "hydrate")]
                                    set_name.set(_v);
                                }
                            />
                        </div>
                        <div class="form-control">
                            <label class="label"><span class="label-text">"Host Path"</span></label>
                            <input
                                type="text"
                                class="input input-bordered w-full"
                                placeholder="/mnt/data"
                                prop:value=move || path.get()
                                on:input=move |ev| {
                                    let _v = event_target_value(&ev);
                                    #[cfg(feature = "hydrate")]
                                    set_path.set(_v);
                                }
                            />
                        </div>
                        <div class="form-control">
                            <label class="label"><span class="label-text">"Discord Role ID"</span></label>
                            <input
                                type="text"
                                class="input input-bordered w-full"
                                placeholder="123456789"
                                prop:value=move || role_id.get()
                                on:input=move |ev| {
                                    let _v = event_target_value(&ev);
                                    #[cfg(feature = "hydrate")]
                                    set_role_id.set(_v);
                                }
                            />
                        </div>
                    </div>
                    <div class="card-actions justify-end mt-4">
                        <button
                            class="btn btn-primary"
                            prop:disabled=add_disabled
                            on:click=move |_| {
                                #[cfg(feature = "hydrate")]
                                {
                                    let n = name.get_untracked();
                                    let p = path.get_untracked();
                                    if let Ok(rid) = role_id.get_untracked().parse::<u64>() {
                                        send_action(ws_handle, filebrowser_types::AdminAction::AddVolume {
                                            name: n,
                                            path: p,
                                            role_id: rid,
                                        });
                                        set_name.set(String::new());
                                        set_path.set(String::new());
                                        set_role_id.set(String::new());
                                    }
                                }
                            }
                        >
                            "Add"
                        </button>
                    </div>
                </div>
            </div>

            // Volume list
            <div class="overflow-x-auto">
                <table class="table table-zebra w-full">
                    <thead>
                        <tr>
                            <th>"ID"</th>
                            <th>"Name"</th>
                            <th>"Host Path"</th>
                            <th>"Role ID"</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || volumes.get().into_iter().map(|(id, vol_name, vol_path, vol_role_id)| {
                            view! {
                                <tr>
                                    <td>{id}</td>
                                    <td>{vol_name}</td>
                                    <td>{vol_path}</td>
                                    <td>{vol_role_id}</td>
                                    <td>
                                        <button
                                            class="btn btn-error btn-sm"
                                            on:click=move |_| {
                                                #[cfg(feature = "hydrate")]
                                                send_action(ws_handle, filebrowser_types::AdminAction::RemoveVolume { id });
                                            }
                                        >
                                            "Delete"
                                        </button>
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>

            {move || volumes.get().is_empty().then(|| view! {
                <p class="text-base-content/70 mt-4">"No volumes configured."</p>
            })}
        </div>
    }
}
