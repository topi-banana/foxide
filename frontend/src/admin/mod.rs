mod role_selector;
mod tokens;
mod volumes;

pub use role_selector::RoleSelector;
pub use tokens::AdminTokensPage;
pub use volumes::AdminVolumesPage;

use leptos::prelude::*;

use crate::ws::WsCtx;

#[component]
pub fn AdminPage() -> impl IntoView {
    let ws = expect_context::<WsCtx>();

    let (roles, set_roles) = signal(Vec::<(u64, String)>::new());
    let (admin_role_id, set_admin_role_id) = signal(None::<u64>);
    let (error, set_error) = signal(None::<String>);
    let (unauthorized, set_unauthorized) = signal(false);

    #[cfg(not(feature = "hydrate"))]
    let _ = (
        &ws,
        set_roles,
        set_admin_role_id,
        set_error,
        set_unauthorized,
    );

    #[cfg(feature = "hydrate")]
    {
        use filebrowser_types::AdminResponse;

        ws.set_on_admin(move |resp| match resp {
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
        });
        on_cleanup(move || ws.clear_on_admin());
    }

    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        if ws.ready.get() {
            ws.send(filebrowser_types::AdminAction::GetRoles);
        }
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
                        <RoleSelector
                            roles=roles
                            selected=Signal::derive(move || admin_role_id.get())
                            on_select=move |_role_id| {
                                #[cfg(feature = "hydrate")]
                                ws.send(filebrowser_types::AdminAction::SetAdminRole { role_id: _role_id });
                            }
                            disabled=Signal::derive(move || unauthorized.get())
                        />
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
