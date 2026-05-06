use std::rc::Rc;

use filebrowser_types::{AdminResponse, BrowseResponse, ClientMsg};
use yew::Callback;

use crate::header::Theme;

/// Application-level state shared across the component tree via Yew context.
///
/// `App` mutates `data` in response to incoming `ServerMsg`s, then re-emits
/// a fresh [`AppCtx`] so that consumers (the page components) see the change.
#[derive(Clone, PartialEq)]
pub struct AppCtx {
    pub data: Rc<AppData>,
    pub theme: Theme,
    pub sidebar_open: bool,

    /// Send a [`ClientMsg`] over the open WebSocket.
    pub send: Callback<ClientMsg>,
    pub set_theme: Callback<Theme>,
    pub set_sidebar_open: Callback<bool>,
}

#[derive(Default, Clone, PartialEq)]
pub struct AppData {
    /// Logged-in username. `None` until `Hello` arrives (or if unauthenticated).
    pub username: Option<String>,
    /// Discord avatar URL. `None` if unauthenticated or no avatar set.
    pub avatar_url: Option<url::Url>,
    /// `true` once the server has replied with `Hello` or `Unauthenticated`.
    pub ready: bool,
    /// Volumes accessible to the current user (populated after Hello).
    pub volumes: Vec<(u64, String)>,

    pub admin: AdminData,
    pub browse: BrowseData,
}

#[derive(Default, Clone, PartialEq)]
pub struct AdminData {
    pub roles: Vec<(u64, String)>,
    pub admin_role_id: Option<u64>,
    pub tokens: Vec<filebrowser_types::TokenInfo>,
    pub volumes: Vec<filebrowser_types::VolumeInfo>,
    pub error: Option<String>,
    pub unauthorized: bool,
    /// Bumped on every admin response so pages can detect new arrivals.
    pub seq: u64,
}

#[derive(Default, Clone, PartialEq)]
pub struct BrowseData {
    pub entries: Vec<filebrowser_types::DirEntry>,
    pub error: Option<String>,
    pub seq: u64,
}

impl AdminData {
    pub fn apply(&mut self, resp: AdminResponse) {
        self.seq = self.seq.wrapping_add(1);
        match resp {
            AdminResponse::Roles {
                roles,
                admin_role_id,
            } => {
                self.roles = roles.into_iter().map(|r| (r.id, r.name)).collect();
                self.admin_role_id = admin_role_id;
                self.error = None;
                self.unauthorized = false;
            }
            AdminResponse::AdminRoleUpdated { role_id } => {
                self.admin_role_id = Some(role_id);
                self.error = None;
            }
            AdminResponse::Tokens { tokens } => {
                self.tokens = tokens;
                self.error = None;
                self.unauthorized = false;
            }
            AdminResponse::Volumes { volumes } => {
                self.volumes = volumes;
                self.error = None;
                self.unauthorized = false;
            }
            AdminResponse::VolumeAdded { volume } => {
                self.volumes.push(volume);
                self.error = None;
            }
            AdminResponse::VolumeRemoved { id } => {
                self.volumes.retain(|v| v.id != id);
                self.error = None;
            }
            AdminResponse::Error { message } => {
                self.error = Some(message);
            }
            AdminResponse::Unauthorized => {
                self.unauthorized = true;
            }
        }
    }
}

impl BrowseData {
    pub fn apply(&mut self, resp: BrowseResponse) {
        self.seq = self.seq.wrapping_add(1);
        match resp {
            BrowseResponse::DirectoryListing { entries } => {
                self.entries = entries;
                self.error = None;
            }
            BrowseResponse::Error { message } => {
                self.entries.clear();
                self.error = Some(message);
            }
        }
    }
}
