use wincode::{SchemaRead, SchemaWrite};

// --- Client → Server ---

#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
pub enum ClientMsg {
    Admin(AdminAction),
    GetMyVolumes,
    Browse(BrowseAction),
}

#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
pub enum BrowseAction {
    ListDirectory { volume_id: u64, path: String },
}

#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
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
    },
    RemoveVolume {
        id: u64,
    },
}

// --- Server → Client ---

#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
pub enum ServerMsg {
    Hello {
        username: String,
        avatar_url: Option<String>,
    },
    Unauthenticated,
    Admin(AdminResponse),
    MyVolumes {
        volumes: Vec<VolumeInfo>,
    },
    Browse(BrowseResponse),
}

#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
pub enum BrowseResponse {
    DirectoryListing { entries: Vec<DirEntry> },
    Error { message: String },
}

#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
pub struct DirEntry {
    pub name: String,
    pub entry_type: EntryType,
    pub size: u64,
}

#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
pub enum EntryType {
    File,
    Directory,
}

#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
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

#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
pub struct RoleInfo {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
pub struct TokenInfo {
    pub user_id: u64,
    pub username: String,
    pub expires: String,
}

#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
pub struct VolumeInfo {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub role_id: u64,
}
