use std::time::Instant;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use tower_cookies::Cookies;

use crate::AppState;
use crate::login::User;
use filebrowser_types::{ClientMsg, ServerMsg};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(ws_handler))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    cookies: Cookies,
) -> impl IntoResponse {
    let user = authenticate(&state, &cookies).await;
    ws.on_upgrade(move |socket| handle_socket(socket, user, state))
}

async fn authenticate(state: &AppState, cookies: &Cookies) -> Option<User> {
    let session = cookies.get("auth-session")?;
    let (expires, user) = state.token_storage.get(session.value()).await?;
    if Instant::now() > expires {
        return None;
    }
    Some(user)
}

async fn handle_socket(mut socket: WebSocket, user: Option<User>, state: AppState) {
    // Send initial auth status
    let msg = match &user {
        Some(u) => ServerMsg::Hello {
            username: u.username.clone(),
        },
        None => ServerMsg::Unauthenticated,
    };
    let bytes = wincode::serialize(&msg).unwrap();
    if socket.send(Message::Binary(bytes.into())).await.is_err() {
        return;
    }

    // Process incoming requests
    while let Some(Ok(msg)) = socket.recv().await {
        let Message::Binary(data) = msg else {
            if matches!(msg, Message::Close(_)) {
                break;
            }
            continue;
        };

        let Ok(client_msg) = wincode::deserialize::<ClientMsg>(&data) else {
            continue;
        };

        let response = match client_msg {
            ClientMsg::Admin(action) => match &user {
                Some(user) => ServerMsg::Admin(crate::admin::handle(&state, user, action).await),
                None => ServerMsg::Unauthenticated,
            },
        };

        let bytes = wincode::serialize(&response).unwrap();
        if socket.send(Message::Binary(bytes.into())).await.is_err() {
            break;
        }
    }
}
