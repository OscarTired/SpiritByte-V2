//! Shared application state holding the (optionally unlocked) vault session.

use std::sync::Mutex;

use crate::vault::{Session, VaultPaths};

pub struct AppState {
    pub paths: VaultPaths,
    pub session: Mutex<Option<Session>>,
}

impl AppState {
    pub fn new(paths: VaultPaths) -> Self {
        Self {
            paths,
            session: Mutex::new(None),
        }
    }
}
