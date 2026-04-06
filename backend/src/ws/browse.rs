use std::path::Path;

use serenity::all::{GuildId, UserId};

use crate::AppState;
use crate::login::User;
use crate::ws::SocketWriter;
use filebrowser_types::{BrowseResponse, DirEntry, EntryType, ServerMsg};

pub async fn list_directory(state: &AppState, user: &User, writer: &SocketWriter, volume_id: u64) {
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

    // Check directory existence and read entries
    let path = Path::new(&volume.path);
    if !path.is_dir() {
        writer.send(ServerMsg::Browse(BrowseResponse::Error {
            message: "Volume directory does not exist".into(),
        }));
        return;
    }

    let mut entries = Vec::new();
    match std::fs::read_dir(path) {
        Ok(read_dir) => {
            for entry in read_dir.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                let entry_type = if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    EntryType::Directory
                } else {
                    EntryType::File
                };
                entries.push(DirEntry { name, entry_type });
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
