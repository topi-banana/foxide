use std::time::Instant;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use tower_cookies::Cookies;

use crate::AppState;
use crate::login::User;
use filebrowser_types::ServerMsg;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(ws_handler))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    cookies: Cookies,
) -> impl IntoResponse {
    let user = authenticate(&state, &cookies).await;
    ws.on_upgrade(move |socket| handle_socket(socket, user))
}

async fn authenticate(state: &AppState, cookies: &Cookies) -> Option<User> {
    let session = cookies.get("auth-session")?;
    let (expires, user) = state.token_storage.get(session.value()).await?;
    if Instant::now() > expires {
        return None;
    }
    Some(user)
}

async fn handle_socket(mut socket: WebSocket, user: Option<User>) {
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

    while let Some(Ok(msg)) = socket.recv().await {
        if matches!(msg, Message::Close(_)) {
            break;
        }
    }
}
