use leptos::prelude::*;

use crate::ws::WsCtx;

#[component]
pub fn AdminTokensPage() -> impl IntoView {
    let ws = expect_context::<WsCtx>();

    let (tokens, set_tokens) = signal(Vec::<(u64, String, String)>::new());
    let (error, set_error) = signal(None::<String>);
    let (unauthorized, set_unauthorized) = signal(false);

    #[cfg(not(feature = "hydrate"))]
    let _ = (&ws, set_tokens, set_error, set_unauthorized);

    #[cfg(feature = "hydrate")]
    {
        use filebrowser_types::AdminResponse;

        ws.set_on_admin(move |resp| match resp {
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
        });
        on_cleanup(move || ws.clear_on_admin());
    }

    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        if ws.ready.get() {
            ws.send(filebrowser_types::AdminAction::GetTokens);
        }
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
