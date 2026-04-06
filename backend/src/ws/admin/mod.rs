mod roles;
mod tokens;
mod volumes;

use serenity::all::{GuildId, UserId};

use crate::AppState;
use crate::login::User;
use crate::ws::SocketWriter;
use filebrowser_types::{AdminAction, AdminResponse, ServerMsg};

/// Check if a user has the admin role in the guild.
/// All admin actions use the same permission check.
async fn is_authorized(state: &AppState, user: &User, _action: &AdminAction) -> bool {
    let Some(admin_role_id) = state.admin_settings.get_admin_role_id().unwrap_or(None) else {
        // Not configured yet — allow any authenticated user
        return true;
    };

    let guild_id = GuildId::new(state.guild_id);
    let user_id = UserId::new(user.user_id);

    match state.bot_http.get_member(guild_id, user_id).await {
        Ok(member) => member.roles.iter().any(|r| r.get() == admin_role_id),
        Err(e) => {
            tracing::error!("failed to fetch guild member: {e}");
            false
        }
    }
}

pub async fn handle(state: &AppState, user: &User, writer: &SocketWriter, action: AdminAction) {
    if !is_authorized(state, user, &action).await {
        writer.send(ServerMsg::Admin(AdminResponse::Unauthorized));
        return;
    }

    match action {
        AdminAction::GetRoles => roles::get_roles(state, writer).await,
        AdminAction::SetAdminRole { role_id } => roles::set_admin_role(state, writer, role_id),
        AdminAction::GetTokens => tokens::get_tokens(state, writer),
        AdminAction::GetVolumes => volumes::get_volumes(state, writer),
        AdminAction::AddVolume {
            name,
            path,
            role_id,
        } => volumes::add_volume(state, writer, name, path, role_id).await,
        AdminAction::RemoveVolume { id } => volumes::remove_volume(state, writer, id).await,
    }
}
