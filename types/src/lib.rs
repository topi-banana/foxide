use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// --- Client → Server ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClientMsg {
    Admin(AdminAction),
    GetMyVolumes,
    Browse(BrowseAction),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BrowseAction {
    ListDirectory {
        volume_id: u64,
        path: String,
    },
    EntryAction {
        volume_id: u64,
        path: String,
        action: EntryActionKind,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryActionKind {
    Download,
    Rename,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AdminAction {
    GetRoles,
    SetAdminRole {
        role_id: u64,
    },
    GetTokens,
    GetVolumes,
    AddVolume {
        name: String,
        path: String,
        role_id: u64,
        permission: Permission,
    },
    RemoveVolume {
        id: u64,
    },
}

// --- Server → Client ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServerMsg {
    Hello {
        username: String,
        avatar_url: Option<url::Url>,
    },
    Unauthenticated,
    Admin(AdminResponse),
    MyVolumes {
        volumes: Vec<VolumeInfo>,
    },
    Browse(BrowseResponse),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BrowseResponse {
    DirectoryListing { entries: Vec<DirEntry> },
    Error { message: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub entry_type: EntryType,
    pub size: u64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntryType {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AdminResponse {
    Roles {
        roles: Vec<RoleInfo>,
        admin_role_id: Option<u64>,
    },
    AdminRoleUpdated {
        role_id: u64,
    },
    Tokens {
        tokens: Vec<TokenInfo>,
    },
    Volumes {
        volumes: Vec<VolumeInfo>,
    },
    VolumeAdded {
        volume: VolumeInfo,
    },
    VolumeRemoved {
        id: u64,
    },
    Error {
        message: String,
    },
    Unauthorized,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleInfo {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenInfo {
    pub user_id: u64,
    pub username: String,
    pub expires: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub role_id: u64,
    pub permission: Permission,
}
