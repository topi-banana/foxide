mod callback;
mod redirect;

use std::path::Path;

use axum::Router;
use axum::routing::get;
use chrono::{DateTime, Days, Utc};
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(redirect::login_redirect))
        .route("/callback", get(callback::login_callback))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: u64,
    pub username: String,
}

pub struct UserStorage {
    db: sled::Db,
}

impl UserStorage {
    pub fn open(path: impl AsRef<Path>) -> sled::Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn insert(&self, user: &User) -> sled::Result<()> {
        let value = serde_json::to_vec(user).expect("failed to serialize User");
        self.db.insert(user.user_id.to_be_bytes(), value)?;
        Ok(())
    }

    pub fn get(&self, user_id: u64) -> sled::Result<Option<User>> {
        let Some(bytes) = self.db.get(user_id.to_be_bytes())? else {
            return Ok(None);
        };
        let user: User = serde_json::from_slice(&bytes).expect("failed to deserialize User");
        Ok(Some(user))
    }
}

pub struct TokenStorage {
    db: sled::Db,
}

#[derive(Serialize, Deserialize)]
struct TokenEntry {
    expires: DateTime<Utc>,
    user: User,
}

impl TokenStorage {
    pub fn open(path: impl AsRef<Path>) -> sled::Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    fn create_new_token() -> (String, [u8; 32]) {
        let token = Alphanumeric.sample_string(&mut rand::rng(), 32);
        let id = Self::get_id_from_token(&token);
        (token, id)
    }

    fn get_id_from_token(token: &str) -> [u8; 32] {
        let mut id = [0; 32];
        id.copy_from_slice(&Sha256::digest(token));
        id
    }

    pub fn create(&self, user: User) -> String {
        loop {
            let (token, id) = Self::create_new_token();
            if self.db.contains_key(id).expect("failed to check token") {
                continue;
            }
            let entry = TokenEntry {
                expires: Utc::now() + Days::new(7),
                user,
            };
            let value = serde_json::to_vec(&entry).expect("failed to serialize TokenEntry");
            self.db.insert(id, value).expect("failed to insert token");
            break token;
        }
    }

    pub fn list_all(&self) -> Vec<(DateTime<Utc>, User)> {
        self.db
            .iter()
            .filter_map(|result| {
                let (_, value) = result.ok()?;
                let entry: TokenEntry = serde_json::from_slice(&value).ok()?;
                Some((entry.expires, entry.user))
            })
            .collect()
    }

    pub fn get(&self, session_token: &str) -> Option<(DateTime<Utc>, User)> {
        let id = Self::get_id_from_token(session_token);
        let bytes = self.db.get(id).expect("failed to get token")?;
        let entry: TokenEntry =
            serde_json::from_slice(&bytes).expect("failed to deserialize TokenEntry");
        Some((entry.expires, entry.user))
    }
}
