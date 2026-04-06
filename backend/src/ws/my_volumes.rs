use serenity::all::{GuildId, UserId};

use crate::AppState;
use crate::login::User;
use crate::ws::SocketWriter;
use filebrowser_types::{ServerMsg, VolumeInfo};

pub async fn get_my_volumes(state: &AppState, user: &User, writer: &SocketWriter) {
    let guild_id = GuildId::new(state.guild_id);
    let user_id = UserId::new(user.user_id);

    let user_role_ids: Vec<u64> = match state.bot_http.get_member(guild_id, user_id).await {
        Ok(member) => member.roles.iter().map(|r| r.get()).collect(),
        Err(e) => {
            tracing::error!("failed to fetch guild member for volume access: {e}");
            vec![]
        }
    };

    let volumes = state
        .volume_storage
        .list_all()
        .into_iter()
        .filter(|v| user_role_ids.contains(&v.role_id))
        .map(|v| VolumeInfo {
            id: v.id,
            name: v.name,
            path: v.path,
            role_id: v.role_id,
        })
        .collect();

    writer.send(ServerMsg::MyVolumes { volumes });
}
