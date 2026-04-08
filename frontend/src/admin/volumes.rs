use filebrowser_types::Permission;
use leptos::prelude::*;

use super::RoleSelector;
use crate::ws::WsCtx;

#[component]
pub fn AdminVolumesPage() -> impl IntoView {
    let ws = expect_context::<WsCtx>();

    let (volumes, set_volumes) = signal(Vec::<(u64, String, String, u64, Permission)>::new());
    let (roles, set_roles) = signal(Vec::<(u64, String)>::new());
    let (error, set_error) = signal(None::<String>);
    let (unauthorized, set_unauthorized) = signal(false);

    // Form fields
    let (name, set_name) = signal(String::new());
    let (path, set_path) = signal(String::new());
    let (selected_role, set_selected_role) = signal(None::<u64>);
    let (selected_permission, set_selected_permission) = signal(Permission::ReadOnly);

    #[cfg(not(feature = "hydrate"))]
    let _ = (
        &ws,
        set_volumes,
        set_roles,
        set_error,
        set_unauthorized,
        set_name,
        set_path,
        set_selected_role,
        set_selected_permission,
    );

    #[cfg(feature = "hydrate")]
    {
        use filebrowser_types::AdminResponse;

        ws.set_on_admin(move |resp| match resp {
            AdminResponse::Volumes { volumes } => {
                set_volumes.set(
                    volumes
                        .into_iter()
                        .map(|v| (v.id, v.name, v.path, v.role_id, v.permission))
                        .collect(),
                );
                set_error.set(None);
                set_unauthorized.set(false);
            }
            AdminResponse::Roles { roles, .. } => {
                set_roles.set(roles.into_iter().map(|r| (r.id, r.name)).collect());
            }
            AdminResponse::VolumeAdded { volume } => {
                set_volumes.update(|vols| {
                    vols.push((
                        volume.id,
                        volume.name,
                        volume.path,
                        volume.role_id,
                        volume.permission,
                    ));
                });
                set_error.set(None);
            }
            AdminResponse::VolumeRemoved { id } => {
                set_volumes.update(|vols| {
                    vols.retain(|(vid, _, _, _, _)| *vid != id);
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
        });
    }

    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        if ws.ready.get() {
            ws.send(filebrowser_types::AdminAction::GetVolumes);
            ws.send(filebrowser_types::AdminAction::GetRoles);
        }
    });

    let add_disabled = move || {
        unauthorized.get()
            || name.get().is_empty()
            || path.get().is_empty()
            || selected_role.get().is_none()
    };

    // Resolve role_id -> role name using the roles list
    let role_name = move |role_id: u64| {
        roles
            .get()
            .iter()
            .find(|(id, _)| *id == role_id)
            .map(|(_, n)| n.clone())
            .unwrap_or_else(|| role_id.to_string())
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
                    <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mt-2">
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
                            <label class="label"><span class="label-text">"Discord Role"</span></label>
                            <RoleSelector
                                roles=roles
                                selected=Signal::derive(move || selected_role.get())
                                on_select=move |role_id| {
                                    set_selected_role.set(Some(role_id));
                                }
                                disabled=Signal::derive(move || unauthorized.get())
                            />
                        </div>
                        <div class="form-control">
                            <label class="label"><span class="label-text">"Permission"</span></label>
                            <select
                                class="select select-bordered w-full"
                                prop:disabled=move || unauthorized.get()
                                on:change=move |ev| {
                                    let _v = event_target_value(&ev);
                                    #[cfg(feature = "hydrate")]
                                    {
                                        let perm = if _v == "rw" { Permission::ReadWrite } else { Permission::ReadOnly };
                                        set_selected_permission.set(perm);
                                    }
                                }
                            >
                                <option value="ro" selected=move || selected_permission.get() == Permission::ReadOnly>"Read Only"</option>
                                <option value="rw" selected=move || selected_permission.get() == Permission::ReadWrite>"Read & Write"</option>
                            </select>
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
                                    if let Some(rid) = selected_role.get_untracked() {
                                        ws.send(filebrowser_types::AdminAction::AddVolume {
                                            name: n,
                                            path: p,
                                            role_id: rid,
                                            permission: selected_permission.get_untracked(),
                                        });
                                        set_name.set(String::new());
                                        set_path.set(String::new());
                                        set_selected_role.set(None);
                                        set_selected_permission.set(Permission::ReadOnly);
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
                            <th>"Role"</th>
                            <th>"Permission"</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || volumes.get().into_iter().map(|(id, vol_name, vol_path, vol_role_id, vol_perm)| {
                            let rname = role_name(vol_role_id);
                            let perm_label = match vol_perm {
                                Permission::ReadOnly => "Read Only",
                                Permission::ReadWrite => "Read & Write",
                            };
                            view! {
                                <tr>
                                    <td>{id}</td>
                                    <td>{vol_name}</td>
                                    <td>{vol_path}</td>
                                    <td>{rname}</td>
                                    <td>{perm_label}</td>
                                    <td>
                                        <button
                                            class="btn btn-error btn-sm"
                                            on:click=move |_| {
                                                #[cfg(feature = "hydrate")]
                                                ws.send(filebrowser_types::AdminAction::RemoveVolume { id });
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
