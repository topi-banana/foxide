use serenity::all::{GuildId, UserId};

use crate::AppState;
use crate::login::User;
use crate::ws::SocketWriter;
use filebrowser_types::{AdminAction, AdminResponse, RoleInfo, ServerMsg};

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
        AdminAction::GetRoles => get_roles(state, writer).await,
        AdminAction::SetAdminRole { role_id } => set_admin_role(state, writer, role_id),
    }
}

async fn get_roles(state: &AppState, writer: &SocketWriter) {
    let guild_id = GuildId::new(state.guild_id);
    match state.bot_http.get_guild_roles(guild_id).await {
        Ok(guild_roles) => {
            let roles = guild_roles
                .into_iter()
                .map(|r| RoleInfo {
                    id: r.id.get(),
                    name: r.name,
                })
                .collect();
            let admin_role_id = state.admin_settings.get_admin_role_id().unwrap_or(None);
            writer.send(ServerMsg::Admin(AdminResponse::Roles {
                roles,
                admin_role_id,
            }));
        }
        Err(e) => {
            tracing::error!("failed to fetch guild roles: {e}");
            writer.send(ServerMsg::Admin(AdminResponse::Error {
                message: "Failed to fetch guild roles".to_string(),
            }));
        }
    }
}

fn set_admin_role(state: &AppState, writer: &SocketWriter, role_id: u64) {
    match state.admin_settings.set_admin_role_id(role_id) {
        Ok(()) => {
            writer.send(ServerMsg::Admin(AdminResponse::AdminRoleUpdated {
                role_id,
            }));
        }
        Err(e) => {
            tracing::error!("failed to set admin role: {e}");
            writer.send(ServerMsg::Admin(AdminResponse::Error {
                message: "Failed to save admin role".to_string(),
            }));
        }
    }
}
