use filebrowser_types::{AdminAction, ClientMsg};
use yew::prelude::*;

use crate::ws::AppCtx;

pub struct AdminTokensPage {
    ctx: AppCtx,
    _ctx_handle: ContextHandle<AppCtx>,
}

pub enum Msg {
    ContextChanged(AppCtx),
}

impl Component for AdminTokensPage {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (app_ctx, handle) = ctx
            .link()
            .context::<AppCtx>(ctx.link().callback(Msg::ContextChanged))
            .expect("AppCtx not provided");

        if app_ctx.data.ready {
            app_ctx.send.emit(ClientMsg::Admin(AdminAction::GetTokens));
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
                    self.ctx.send.emit(ClientMsg::Admin(AdminAction::GetTokens));
                }
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
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

        let rows: Html = admin
            .tokens
            .iter()
            .map(|t| {
                html! {
                    <tr>
                        <td>{t.user_id}</td>
                        <td>{&t.username}</td>
                        <td>{t.expires.format("%Y-%m-%d %H:%M:%S").to_string()}</td>
                    </tr>
                }
            })
            .collect();

        let empty = admin.tokens.is_empty().then(|| {
            html! {
                <p class="text-base-content/70 mt-4">{"No active sessions."}</p>
            }
        });

        html! {
            <div class="max-w-4xl mx-auto">
                <h1 class="text-2xl font-bold mb-6">{"Active Sessions"}</h1>
                {unauthorized}
                {error}
                <div class="overflow-x-auto">
                    <table class="table table-zebra w-full">
                        <thead>
                            <tr>
                                <th>{"User ID"}</th>
                                <th>{"Username"}</th>
                                <th>{"Expires"}</th>
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
