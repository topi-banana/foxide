use std::path::Path;

use serenity::all::{GuildId, UserId};

use crate::AppState;
use crate::login::User;
use filebrowser_types::{AdminAction, AdminResponse, RoleInfo};

/// Persistent admin settings backed by sled.
pub struct AdminSettings {
    db: sled::Db,
}

impl AdminSettings {
    pub fn open(path: impl AsRef<Path>) -> sled::Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn get_admin_role_id(&self) -> sled::Result<Option<u64>> {
        let Some(bytes) = self.db.get(b"admin_role_id")? else {
            return Ok(None);
        };
        let arr: [u8; 8] = bytes.as_ref().try_into().expect("invalid admin_role_id");
        Ok(Some(u64::from_be_bytes(arr)))
    }

    pub fn set_admin_role_id(&self, role_id: u64) -> sled::Result<()> {
        self.db.insert(b"admin_role_id", &role_id.to_be_bytes())?;
        Ok(())
    }
}

/// Check if a user has the admin role in the guild.
/// All admin actions use the same permission check.
async fn is_authorized(state: &AppState, user: &User, _action: &AdminAction) -> bool {
    let Some(admin_role_id) = state.admin_settings.get_admin_role_id().unwrap_or(None) else {
        return false;
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

/// Handle an admin action, returning a response.
pub async fn handle(state: &AppState, user: &User, action: AdminAction) -> AdminResponse {
    if !is_authorized(state, user, &action).await {
        return AdminResponse::Unauthorized;
    }

    match action {
        AdminAction::GetRoles => get_roles(state).await,
        AdminAction::SetAdminRole { role_id } => set_admin_role(state, role_id),
    }
}

async fn get_roles(state: &AppState) -> AdminResponse {
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
            AdminResponse::Roles {
                roles,
                admin_role_id,
            }
        }
        Err(e) => {
            tracing::error!("failed to fetch guild roles: {e}");
            AdminResponse::Error {
                message: "Failed to fetch guild roles".to_string(),
            }
        }
    }
}

fn set_admin_role(state: &AppState, role_id: u64) -> AdminResponse {
    match state.admin_settings.set_admin_role_id(role_id) {
        Ok(()) => AdminResponse::AdminRoleUpdated { role_id },
        Err(e) => {
            tracing::error!("failed to set admin role: {e}");
            AdminResponse::Error {
                message: "Failed to save admin role".to_string(),
            }
        }
    }
}
