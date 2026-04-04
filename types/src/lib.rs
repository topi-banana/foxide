use wincode::{SchemaRead, SchemaWrite};

// --- Client → Server ---

#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
pub enum ClientMsg {
    Admin(AdminAction),
}

#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
pub enum AdminAction {
    GetRoles,
    SetAdminRole { role_id: u64 },
}

// --- Server → Client ---

#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
pub enum ServerMsg {
    Hello { username: String },
    Unauthenticated,
    Admin(AdminResponse),
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
