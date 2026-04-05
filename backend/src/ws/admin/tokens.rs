use crate::AppState;
use crate::ws::SocketWriter;
use filebrowser_types::{AdminResponse, ServerMsg, TokenInfo};

pub fn get_tokens(state: &AppState, writer: &SocketWriter) {
    let tokens = state
        .token_storage
        .list_all()
        .into_iter()
        .map(|(expires, user)| TokenInfo {
            user_id: user.user_id,
            username: user.username,
            expires: expires.to_rfc3339(),
        })
        .collect();
    writer.send(ServerMsg::Admin(AdminResponse::Tokens { tokens }));
}
