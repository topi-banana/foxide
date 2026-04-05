use serenity::all::GuildId;

use crate::AppState;
use crate::ws::SocketWriter;
use filebrowser_types::{AdminResponse, RoleInfo, ServerMsg};

pub async fn get_roles(state: &AppState, writer: &SocketWriter) {
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

pub fn set_admin_role(state: &AppState, writer: &SocketWriter, role_id: u64) {
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
