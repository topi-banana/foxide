use std::path::Path;

use filebrowser_types::Permission;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub role_id: u64,
    #[serde(default = "default_permission")]
    pub permission: Permission,
}

fn default_permission() -> Permission {
    Permission::ReadOnly
}

/// Persistent volume configuration backed by sled.
pub struct VolumeStorage {
    db: sled::Db,
}

impl VolumeStorage {
    pub fn open(path: impl AsRef<Path>) -> sled::Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn list_all(&self) -> Vec<Volume> {
        self.db
            .iter()
            .filter_map(|entry| {
                let (_, value) = entry.ok()?;
                serde_json::from_slice(&value).ok()
            })
            .collect()
    }

    pub fn add(
        &self,
        name: String,
        path: String,
        role_id: u64,
        permission: Permission,
    ) -> sled::Result<Volume> {
        let id = self.db.generate_id()?;
        let volume = Volume {
            id,
            name,
            path,
            role_id,
            permission,
        };
        let json = serde_json::to_vec(&volume).expect("failed to serialize volume");
        self.db.insert(id.to_be_bytes(), json)?;
        Ok(volume)
    }

    pub fn remove(&self, id: u64) -> sled::Result<bool> {
        let removed = self.db.remove(id.to_be_bytes())?;
        Ok(removed.is_some())
    }
}
