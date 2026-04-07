mod admin;
mod browse;
pub(crate) mod my_volumes;

use std::collections::BTreeMap;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tower_cookies::Cookies;

use crate::AppState;
use crate::login::User;
use filebrowser_types::{ClientMsg, ServerMsg};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(ws_handler))
}

#[derive(Clone)]
pub struct SocketWriter {
    tx: mpsc::UnboundedSender<ServerMsg>,
}

impl SocketWriter {
    pub fn send(&self, msg: ServerMsg) -> bool {
        match self.tx.send(msg) {
            Ok(_) => true,
            Err(e) => {
                tracing::error!("Failed to send message: {:?}", e);
                false
            }
        }
    }
}

/// Holds a sender per WebSocket connection, keyed by user_id.
/// One user may have multiple connections (multiple tabs, etc.).
pub struct SocketStorage {
    senders: Mutex<BTreeMap<u64, Vec<SocketWriter>>>,
}

impl SocketStorage {
    pub fn new() -> Self {
        Self {
            senders: Mutex::new(BTreeMap::new()),
        }
    }

    async fn insert(&self, user_id: u64, tx: SocketWriter) {
        self.senders
            .lock()
            .await
            .entry(user_id)
            .or_default()
            .push(tx);
    }

    async fn remove(&self, user_id: u64, tx_ptr: usize) {
        let mut map = self.senders.lock().await;
        if let Some(senders) = map.get_mut(&user_id) {
            senders.retain(|s| std::ptr::from_ref(s) as usize != tx_ptr);
            if senders.is_empty() {
                map.remove(&user_id);
            }
        }
    }

    /// Send a message to all connections of a user.
    pub async fn send_to(&self, user_id: u64, msg: &ServerMsg) {
        let mut map = self.senders.lock().await;
        if let Some(senders) = map.get_mut(&user_id) {
            senders.retain(|tx| tx.send(msg.clone()));
            if senders.is_empty() {
                map.remove(&user_id);
            }
        }
    }

    /// Return all currently connected user IDs.
    pub async fn connected_user_ids(&self) -> Vec<u64> {
        self.senders.lock().await.keys().copied().collect()
    }
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
    let (expires, user) = state.token_storage.get(session.value())?;
    if Utc::now() > expires {
        return None;
    }
    Some(user)
}

async fn handle_socket(socket: WebSocket, user: Option<User>, state: AppState) {
    let (mut sink, mut stream) = socket.split();

    // Send initial auth status
    let msg = match &user {
        Some(u) => ServerMsg::Hello {
            username: u.username.clone(),
            avatar_url: u.avatar_url(),
        },
        None => ServerMsg::Unauthenticated,
    };
    let bytes = rmp_serde::to_vec(&msg).unwrap();
    if sink.send(Message::Binary(bytes.into())).await.is_err() {
        return;
    }

    // Set up channel and register in storage if authenticated
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMsg>();
    let writer = SocketWriter { tx };
    let tx_ptr = std::ptr::from_ref(&writer) as usize;
    if let Some(ref user) = user {
        state
            .socket_storage
            .insert(user.user_id, writer.clone())
            .await;
    }

    // Spawn a task that forwards channel messages to the WebSocket sink
    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let bytes = rmp_serde::to_vec(&msg).unwrap();
            if sink.send(Message::Binary(bytes.into())).await.is_err() {
                break;
            }
        }
    });

    // Process incoming requests
    while let Some(Ok(msg)) = stream.next().await {
        let Message::Binary(data) = msg else {
            if matches!(msg, Message::Close(_)) {
                break;
            }
            continue;
        };

        let Ok(client_msg) = rmp_serde::from_slice::<ClientMsg>(&data) else {
            continue;
        };

        match client_msg {
            ClientMsg::Admin(action) => match &user {
                Some(user) => admin::handle(&state, user, &writer, action).await,
                None => {
                    writer.send(ServerMsg::Unauthenticated);
                }
            },
            ClientMsg::GetMyVolumes => match &user {
                Some(user) => my_volumes::get_my_volumes(&state, user, &writer).await,
                None => {
                    writer.send(ServerMsg::Unauthenticated);
                }
            },
            ClientMsg::Browse(action) => match &user {
                Some(user) => {
                    use filebrowser_types::BrowseAction;
                    match action {
                        BrowseAction::ListDirectory { volume_id, path } => {
                            browse::list_directory(&state, user, &writer, volume_id, &path).await;
                        }
                    }
                }
                None => {
                    writer.send(ServerMsg::Unauthenticated);
                }
            },
        };
    }

    // Cleanup
    if let Some(ref user) = user {
        state.socket_storage.remove(user.user_id, tx_ptr).await;
    }
    drop(writer);
    let _ = write_task.await;
}
