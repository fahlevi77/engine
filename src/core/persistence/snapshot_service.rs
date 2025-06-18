use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::util::state_holder::StateHolder;
use crate::core::util::{from_bytes, to_bytes};

use super::persistence_store::PersistenceStore;

/// Basic snapshot service keeping arbitrary state bytes.
#[derive(Default)]
pub struct SnapshotService {
    state: Mutex<Vec<u8>>, // serialized runtime state
    pub persistence_store: Option<Arc<dyn PersistenceStore>>,
    pub siddhi_app_id: String,
    state_holders: Mutex<HashMap<String, Arc<dyn StateHolder>>>,
}

#[derive(Serialize, Deserialize, Default)]
struct SnapshotData {
    main: Vec<u8>,
    holders: HashMap<String, Vec<u8>>,
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
            state_holders: Mutex::new(HashMap::new()),
        }
    }

    /// Replace the current internal state.
    pub fn set_state(&self, data: Vec<u8>) {
        *self.state.lock().unwrap() = data;
    }

    /// Register a state holder to be included in snapshots.
    pub fn register_state_holder(&self, id: String, holder: Arc<dyn StateHolder>) {
        self.state_holders.lock().unwrap().insert(id, holder);
    }

    /// Retrieve a copy of the internal state.
    pub fn snapshot(&self) -> Vec<u8> {
        self.state.lock().unwrap().clone()
    }

    /// Persist the current state via the configured store.
    pub fn persist(&self) -> Result<String, String> {
        let mut holders = HashMap::new();
        for (id, holder) in self.state_holders.lock().unwrap().iter() {
            holders.insert(id.clone(), holder.snapshot_state());
        }
        let snapshot = SnapshotData {
            main: self.snapshot(),
            holders,
        };
        let data = to_bytes(&snapshot).map_err(|e| e.to_string())?;
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
            let snap: SnapshotData = from_bytes(&data).map_err(|e| e.to_string())?;
            self.set_state(snap.main);
            for (id, bytes) in snap.holders {
                if let Some(holder) = self.state_holders.lock().unwrap().get(&id) {
                    holder.restore_state(&bytes);
                }
            }
            Ok(())
        } else {
            Err("Revision not found".into())
        }
    }
}
