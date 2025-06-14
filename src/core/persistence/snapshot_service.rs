use chrono::Utc;
use std::sync::{Arc, Mutex};

use super::persistence_store::PersistenceStore;

/// Basic snapshot service keeping arbitrary state bytes.
#[derive(Default)]
pub struct SnapshotService {
    state: Mutex<Vec<u8>>, // serialized runtime state
    pub persistence_store: Option<Arc<dyn PersistenceStore>>,
    pub siddhi_app_id: String,
}

impl std::fmt::Debug for SnapshotService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnapshotService")
            .field("siddhi_app_id", &self.siddhi_app_id)
            .finish()
    }
}

impl SnapshotService {
    pub fn new(siddhi_app_id: String) -> Self {
        Self {
            state: Mutex::new(Vec::new()),
            persistence_store: None,
            siddhi_app_id,
        }
    }

    /// Replace the current internal state.
    pub fn set_state(&self, data: Vec<u8>) {
        *self.state.lock().unwrap() = data;
    }

    /// Retrieve a copy of the internal state.
    pub fn snapshot(&self) -> Vec<u8> {
        self.state.lock().unwrap().clone()
    }

    /// Persist the current state via the configured store.
    pub fn persist(&self) -> Result<String, String> {
        let data = self.snapshot();
        let store = self
            .persistence_store
            .as_ref()
            .ok_or("No persistence store")?;
        let revision = Utc::now().timestamp_millis().to_string();
        store.save(&self.siddhi_app_id, &revision, &data);
        Ok(revision)
    }

    /// Load the given revision from the store and set as current state.
    pub fn restore_revision(&self, revision: &str) -> Result<(), String> {
        let store = self
            .persistence_store
            .as_ref()
            .ok_or("No persistence store")?;
        if let Some(data) = store.load(&self.siddhi_app_id, revision) {
            self.set_state(data);
            Ok(())
        } else {
            Err("Revision not found".into())
        }
    }
}
