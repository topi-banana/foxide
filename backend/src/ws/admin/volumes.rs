use crate::AppState;
use crate::ws::SocketWriter;
use crate::ws::my_volumes;
use filebrowser_types::{AdminResponse, Permission, ServerMsg, VolumeInfo};

pub fn get_volumes(state: &AppState, writer: &SocketWriter) {
    let volumes = state
        .volume_storage
        .list_all()
        .into_iter()
        .map(|v| VolumeInfo {
            id: v.id,
            name: v.name,
            path: v.path,
            role_id: v.role_id,
            permission: v.permission,
        })
        .collect();
    writer.send(ServerMsg::Admin(AdminResponse::Volumes { volumes }));
}

pub async fn add_volume(
    state: &AppState,
    writer: &SocketWriter,
    name: String,
    path: String,
    role_id: u64,
    permission: Permission,
) {
    match state.volume_storage.add(name, path, role_id, permission) {
        Ok(v) => {
            writer.send(ServerMsg::Admin(AdminResponse::VolumeAdded {
                volume: VolumeInfo {
                    id: v.id,
                    name: v.name,
                    path: v.path,
                    role_id: v.role_id,
                    permission: v.permission,
                },
            }));
            my_volumes::broadcast_volumes(state).await;
        }
        Err(e) => {
            tracing::error!("failed to add volume: {e}");
            writer.send(ServerMsg::Admin(AdminResponse::Error {
                message: "Failed to add volume".to_string(),
            }));
        }
    }
}

pub async fn remove_volume(state: &AppState, writer: &SocketWriter, id: u64) {
    match state.volume_storage.remove(id) {
        Ok(true) => {
            writer.send(ServerMsg::Admin(AdminResponse::VolumeRemoved { id }));
            my_volumes::broadcast_volumes(state).await;
        }
        Ok(false) => {
            writer.send(ServerMsg::Admin(AdminResponse::Error {
                message: "Volume not found".to_string(),
            }));
        }
        Err(e) => {
            tracing::error!("failed to remove volume: {e}");
            writer.send(ServerMsg::Admin(AdminResponse::Error {
                message: "Failed to remove volume".to_string(),
            }));
        }
    }
}
