use std::path::Path;

use serenity::all::{GuildId, UserId};

use crate::AppState;
use crate::login::User;
use crate::ws::SocketWriter;
use filebrowser_types::{BrowseResponse, DirEntry, EntryType, ServerMsg};

pub async fn list_directory(
    state: &AppState,
    user: &User,
    writer: &SocketWriter,
    volume_id: u64,
    sub_path: &str,
) {
    // Look up the volume
    let volume = match state
        .volume_storage
        .list_all()
        .into_iter()
        .find(|v| v.id == volume_id)
    {
        Some(v) => v,
        None => {
            writer.send(ServerMsg::Browse(BrowseResponse::Error {
                message: "Volume not found".into(),
            }));
            return;
        }
    };

    // Verify the user has the required role
    let guild_id = GuildId::new(state.guild_id);
    let user_id = UserId::new(user.user_id);
    let has_access = match state.bot_http.get_member(guild_id, user_id).await {
        Ok(member) => member.roles.iter().any(|r| r.get() == volume.role_id),
        Err(e) => {
            tracing::error!("failed to fetch guild member for browse access: {e}");
            false
        }
    };

    if !has_access {
        writer.send(ServerMsg::Browse(BrowseResponse::Error {
            message: "Access denied".into(),
        }));
        return;
    }

    // Resolve the target directory, preventing path traversal
    let root = match Path::new(&volume.path).canonicalize() {
        Ok(p) => p,
        Err(_) => {
            writer.send(ServerMsg::Browse(BrowseResponse::Error {
                message: "Volume directory does not exist".into(),
            }));
            return;
        }
    };

    let target = match root.join(sub_path.trim_start_matches('/')).canonicalize() {
        Ok(p) => p,
        Err(_) => {
            writer.send(ServerMsg::Browse(BrowseResponse::Error {
                message: "Directory not found".into(),
            }));
            return;
        }
    };

    if !target.starts_with(&root) {
        writer.send(ServerMsg::Browse(BrowseResponse::Error {
            message: "Access denied".into(),
        }));
        return;
    }

    if !target.is_dir() {
        writer.send(ServerMsg::Browse(BrowseResponse::Error {
            message: "Not a directory".into(),
        }));
        return;
    }

    let mut entries = Vec::new();
    match std::fs::read_dir(&target) {
        Ok(read_dir) => {
            for entry in read_dir.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                let metadata = entry.metadata().ok();
                let entry_type = if metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false) {
                    EntryType::Directory
                } else {
                    EntryType::File
                };
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                let created_at = metadata
                    .as_ref()
                    .and_then(|m| m.created().ok())
                    .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());
                let updated_at = metadata
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());
                entries.push(DirEntry {
                    name,
                    entry_type,
                    size,
                    created_at,
                    updated_at,
                });
            }
        }
        Err(e) => {
            writer.send(ServerMsg::Browse(BrowseResponse::Error {
                message: format!("Failed to read directory: {e}"),
            }));
            return;
        }
    }

    // Sort: directories first, then alphabetically
    entries.sort_by(|a, b| {
        let type_ord = matches!(b.entry_type, EntryType::Directory)
            .cmp(&matches!(a.entry_type, EntryType::Directory));
        type_ord.then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    writer.send(ServerMsg::Browse(BrowseResponse::DirectoryListing {
        entries,
    }));
}
