use filebrowser_types::{AdminAction, ClientMsg, Permission};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use super::role_selector::RoleSelector;
use crate::ws::AppCtx;

pub struct AdminVolumesPage {
    ctx: AppCtx,
    _ctx_handle: ContextHandle<AppCtx>,
    name: String,
    path: String,
    selected_role: Option<u64>,
    selected_permission: Permission,
}

pub enum Msg {
    ContextChanged(AppCtx),
    NameInput(String),
    PathInput(String),
    SelectRole(u64),
    SelectPermission(Permission),
    Add,
    Remove(u64),
}

impl Component for AdminVolumesPage {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (app_ctx, handle) = ctx
            .link()
            .context::<AppCtx>(ctx.link().callback(Msg::ContextChanged))
            .expect("AppCtx not provided");

        if app_ctx.data.ready {
            app_ctx.send.emit(ClientMsg::Admin(AdminAction::GetVolumes));
            app_ctx.send.emit(ClientMsg::Admin(AdminAction::GetRoles));
        }

        Self {
            ctx: app_ctx,
            _ctx_handle: handle,
            name: String::new(),
            path: String::new(),
            selected_role: None,
            selected_permission: Permission::ReadOnly,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ContextChanged(c) => {
                let was_not_ready = !self.ctx.data.ready;
                let now_ready = c.data.ready;
                self.ctx = c;
                if was_not_ready && now_ready {
                    self.ctx
                        .send
                        .emit(ClientMsg::Admin(AdminAction::GetVolumes));
                    self.ctx.send.emit(ClientMsg::Admin(AdminAction::GetRoles));
                }
                true
            }
            Msg::NameInput(v) => {
                self.name = v;
                true
            }
            Msg::PathInput(v) => {
                self.path = v;
                true
            }
            Msg::SelectRole(id) => {
                self.selected_role = Some(id);
                true
            }
            Msg::SelectPermission(p) => {
                self.selected_permission = p;
                true
            }
            Msg::Add => {
                if let Some(role_id) = self.selected_role
                    && !self.name.is_empty()
                    && !self.path.is_empty()
                {
                    self.ctx.send.emit(ClientMsg::Admin(AdminAction::AddVolume {
                        name: self.name.clone(),
                        path: self.path.clone(),
                        role_id,
                        permission: self.selected_permission,
                    }));
                    self.name.clear();
                    self.path.clear();
                    self.selected_role = None;
                    self.selected_permission = Permission::ReadOnly;
                }
                true
            }
            Msg::Remove(id) => {
                self.ctx
                    .send
                    .emit(ClientMsg::Admin(AdminAction::RemoveVolume { id }));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let admin = &self.ctx.data.admin;

        let unauthorized = admin.unauthorized.then(|| {
            html! {
                <div class="alert alert-error mb-4">
                    <span>{"You are not authorized to access this page."}</span>
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

        let on_name = ctx.link().callback(|ev: InputEvent| {
            let target: HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
            Msg::NameInput(target.value())
        });
        let on_path = ctx.link().callback(|ev: InputEvent| {
            let target: HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
            Msg::PathInput(target.value())
        });
        let on_role = ctx.link().callback(Msg::SelectRole);
        let on_permission = ctx.link().callback(|ev: Event| {
            let target: HtmlSelectElement = ev.target().unwrap().dyn_into().unwrap();
            let perm = if target.value() == "rw" {
                Permission::ReadWrite
            } else {
                Permission::ReadOnly
            };
            Msg::SelectPermission(perm)
        });
        let on_add = ctx.link().callback(|_| Msg::Add);

        let add_disabled = admin.unauthorized
            || self.name.is_empty()
            || self.path.is_empty()
            || self.selected_role.is_none();

        let role_name = |role_id: u64| -> String {
            admin
                .roles
                .iter()
                .find(|(id, _)| *id == role_id)
                .map(|(_, n)| n.clone())
                .unwrap_or_else(|| role_id.to_string())
        };

        let rows: Html = admin
            .volumes
            .iter()
            .map(|v| {
                let id = v.id;
                let on_remove = ctx.link().callback(move |_| Msg::Remove(id));
                let perm_label = match v.permission {
                    Permission::ReadOnly => "Read Only",
                    Permission::ReadWrite => "Read & Write",
                };
                html! {
                    <tr>
                        <td>{v.id}</td>
                        <td>{&v.name}</td>
                        <td>{&v.path}</td>
                        <td>{role_name(v.role_id)}</td>
                        <td>{perm_label}</td>
                        <td>
                            <button class="btn btn-error btn-sm" onclick={on_remove}>{"Delete"}</button>
                        </td>
                    </tr>
                }
            })
            .collect();

        let empty = admin.volumes.is_empty().then(|| {
            html! {
                <p class="text-base-content/70 mt-4">{"No volumes configured."}</p>
            }
        });

        html! {
            <div class="max-w-4xl mx-auto">
                <h1 class="text-2xl font-bold mb-6">{"Volumes"}</h1>
                {unauthorized}
                {error}

                <div class="card bg-base-200 shadow-xl mb-6">
                    <div class="card-body">
                        <h2 class="card-title">{"Add Volume"}</h2>
                        <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mt-2">
                            <div class="form-control">
                                <label class="label"><span class="label-text">{"Name"}</span></label>
                                <input
                                    type="text"
                                    class="input input-bordered w-full"
                                    placeholder="My Files"
                                    value={self.name.clone()}
                                    oninput={on_name}
                                />
                            </div>
                            <div class="form-control">
                                <label class="label"><span class="label-text">{"Host Path"}</span></label>
                                <input
                                    type="text"
                                    class="input input-bordered w-full"
                                    placeholder="/mnt/data"
                                    value={self.path.clone()}
                                    oninput={on_path}
                                />
                            </div>
                            <div class="form-control">
                                <label class="label"><span class="label-text">{"Discord Role"}</span></label>
                                <RoleSelector
                                    roles={admin.roles.clone()}
                                    selected={self.selected_role}
                                    on_select={on_role}
                                    disabled={admin.unauthorized}
                                />
                            </div>
                            <div class="form-control">
                                <label class="label"><span class="label-text">{"Permission"}</span></label>
                                <select
                                    class="select select-bordered w-full"
                                    disabled={admin.unauthorized}
                                    onchange={on_permission}
                                >
                                    <option value="ro" selected={self.selected_permission == Permission::ReadOnly}>{"Read Only"}</option>
                                    <option value="rw" selected={self.selected_permission == Permission::ReadWrite}>{"Read & Write"}</option>
                                </select>
                            </div>
                        </div>
                        <div class="card-actions justify-end mt-4">
                            <button class="btn btn-primary" disabled={add_disabled} onclick={on_add}>{"Add"}</button>
                        </div>
                    </div>
                </div>

                <div class="overflow-x-auto">
                    <table class="table table-zebra w-full">
                        <thead>
                            <tr>
                                <th>{"ID"}</th>
                                <th>{"Name"}</th>
                                <th>{"Host Path"}</th>
                                <th>{"Role"}</th>
                                <th>{"Permission"}</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>{rows}</tbody>
                    </table>
                </div>
                {empty}
            </div>
        }
    }
}
