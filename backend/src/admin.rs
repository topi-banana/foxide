use std::path::Path;

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
