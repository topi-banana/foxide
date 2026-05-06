pub mod role_selector;
pub mod tokens;
pub mod volumes;

pub use tokens::AdminTokensPage;
pub use volumes::AdminVolumesPage;

use foxide_types::{AdminAction, ClientMsg};
use yew::prelude::*;

use self::role_selector::RoleSelector;
use crate::ws::AppCtx;

pub struct AdminPage {
    ctx: AppCtx,
    _ctx_handle: ContextHandle<AppCtx>,
}

pub enum Msg {
    ContextChanged(AppCtx),
    SelectRole(u64),
}

impl Component for AdminPage {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (app_ctx, handle) = ctx
            .link()
            .context::<AppCtx>(ctx.link().callback(Msg::ContextChanged))
            .expect("AppCtx not provided");

        if app_ctx.data.ready {
            app_ctx.send.emit(ClientMsg::Admin(AdminAction::GetRoles));
        }

        Self {
            ctx: app_ctx,
            _ctx_handle: handle,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ContextChanged(c) => {
                let was_not_ready = !self.ctx.data.ready;
                let now_ready = c.data.ready;
                self.ctx = c;
                if was_not_ready && now_ready {
                    self.ctx.send.emit(ClientMsg::Admin(AdminAction::GetRoles));
                }
                true
            }
            Msg::SelectRole(role_id) => {
                self.ctx
                    .send
                    .emit(ClientMsg::Admin(AdminAction::SetAdminRole { role_id }));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let admin = &self.ctx.data.admin;

        let unauthorized = admin.unauthorized.then(|| {
            html! {
                <div class="alert alert-error mb-4">
                    <span>{"You are not authorized to access admin settings."}</span>
                </div>
            }
        });

        let error = admin.error.as_ref().map(|msg| {
            html! {
                <div class="alert alert-warning mb-4">
                    <span>{msg}</span>
                </div>
            }
        });

        let on_select_role = ctx.link().callback(Msg::SelectRole);

        html! {
            <div class="max-w-2xl mx-auto">
                <h1 class="text-2xl font-bold mb-6">{"Admin Settings"}</h1>
                {unauthorized}
                {error}

                <div class="card bg-base-200 shadow-xl">
                    <div class="card-body">
                        <h2 class="card-title">{"Admin Role"}</h2>
                        <p class="text-base-content/70">{"Select which Discord role grants admin access."}</p>
                        <div class="form-control w-full mt-4">
                            <RoleSelector
                                roles={admin.roles.clone()}
                                selected={admin.admin_role_id}
                                on_select={on_select_role}
                                disabled={admin.unauthorized}
                            />
                        </div>
                    </div>
                </div>

                <div class="card bg-base-200 shadow-xl mt-6">
                    <div class="card-body">
                        <h2 class="card-title">{"Management"}</h2>
                        <ul class="menu">
                            <li><a href="/admin/tokens">{"Active Sessions"}</a></li>
                            <li><a href="/admin/volumes">{"Volumes"}</a></li>
                        </ul>
                    </div>
                </div>
            </div>
        }
    }
}
