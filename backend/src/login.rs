mod callback;
mod redirect;

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::path::Path;
use std::time::{Duration, Instant};

use axum::Router;
use axum::routing::get;
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

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
    storage: Mutex<BTreeMap<[u8; 32], (Instant, User)>>,
}

impl TokenStorage {
    pub fn new() -> Self {
        Self {
            storage: Mutex::new(BTreeMap::new()),
        }
    }
}

impl TokenStorage {
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

    pub async fn create(&self, user: User) -> String {
        loop {
            let (token, id) = Self::create_new_token();
            let mut storage = self.storage.lock().await;
            if let Entry::Vacant(e) = storage.entry(id) {
                e.insert((Instant::now() + Duration::from_hours(24 * 7), user));
                break token;
            }
        }
    }

    pub async fn get(&self, session_token: &str) -> Option<(Instant, User)> {
        let id = Self::get_id_from_token(session_token);
        self.storage.lock().await.get(&id).cloned()
    }
}
