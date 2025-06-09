use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Trait for simple persistence stores that save full snapshots.
pub trait PersistenceStore: Send + Sync {
    fn save(&self, siddhi_app_id: &str, revision: &str, snapshot: &[u8]);
    fn load(&self, siddhi_app_id: &str, revision: &str) -> Option<Vec<u8>>;
    fn get_last_revision(&self, siddhi_app_id: &str) -> Option<String>;
    fn clear_all_revisions(&self, siddhi_app_id: &str);
}

/// Trait for incremental persistence stores.
pub trait IncrementalPersistenceStore: Send + Sync {
    fn save(&self, revision: &str, snapshot: &[u8]);
    fn load(&self, revision: &str) -> Option<Vec<u8>>;
    fn get_last_revision(&self, siddhi_app_id: &str) -> Option<String>;
    fn clear_all_revisions(&self, siddhi_app_id: &str);
}

/// Very small in-memory implementation useful for tests.
#[derive(Default)]
pub struct InMemoryPersistenceStore {
    inner: Mutex<HashMap<String, HashMap<String, Vec<u8>>>>,
    last_revision: Mutex<HashMap<String, String>>,
}

impl InMemoryPersistenceStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PersistenceStore for InMemoryPersistenceStore {
    fn save(&self, siddhi_app_id: &str, revision: &str, snapshot: &[u8]) {
        let mut m = self.inner.lock().unwrap();
        let entry = m.entry(siddhi_app_id.to_string()).or_default();
        entry.insert(revision.to_string(), snapshot.to_vec());
        self.last_revision
            .lock()
            .unwrap()
            .insert(siddhi_app_id.to_string(), revision.to_string());
    }

    fn load(&self, siddhi_app_id: &str, revision: &str) -> Option<Vec<u8>> {
        self.inner
            .lock()
            .unwrap()
            .get(siddhi_app_id)
            .and_then(|m| m.get(revision).cloned())
    }

    fn get_last_revision(&self, siddhi_app_id: &str) -> Option<String> {
        self.last_revision
            .lock()
            .unwrap()
            .get(siddhi_app_id)
            .cloned()
    }

    fn clear_all_revisions(&self, siddhi_app_id: &str) {
        self.inner.lock().unwrap().remove(siddhi_app_id);
        self.last_revision.lock().unwrap().remove(siddhi_app_id);
    }
}

impl IncrementalPersistenceStore for InMemoryPersistenceStore {
    fn save(&self, revision: &str, snapshot: &[u8]) {
        // For tests we treat incremental same as full with siddhi_app_id="default"
        <Self as PersistenceStore>::save(self, "default", revision, snapshot);
    }

    fn load(&self, revision: &str) -> Option<Vec<u8>> {
        <Self as PersistenceStore>::load(self, "default", revision)
    }

    fn get_last_revision(&self, siddhi_app_id: &str) -> Option<String> {
        PersistenceStore::get_last_revision(self, siddhi_app_id)
    }

    fn clear_all_revisions(&self, siddhi_app_id: &str) {
        PersistenceStore::clear_all_revisions(self, siddhi_app_id)
    }
}
