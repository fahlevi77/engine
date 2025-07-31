use std::cell::RefCell;
use std::collections::HashMap;

// Thread local flag used to request a full snapshot.
thread_local! {
    static REQUEST_FULL: RefCell<bool> = RefCell::new(false);
}

/// Enable or disable full snapshot for the current thread.
pub fn request_for_full_snapshot(enable: bool) {
    REQUEST_FULL.with(|f| *f.borrow_mut() = enable);
}

/// Whether the current thread requested a full snapshot.
pub fn is_request_for_full_snapshot() -> bool {
    REQUEST_FULL.with(|f| *f.borrow())
}

/// Serialized incremental snapshot information placeholder.
#[derive(Debug, Default, Clone)]
pub struct IncrementalSnapshot {
    pub incremental_state: HashMap<String, HashMap<String, Vec<u8>>>,
    pub incremental_state_base: HashMap<String, HashMap<String, Vec<u8>>>,
    pub periodic_state: HashMap<String, HashMap<String, Vec<u8>>>,
}

/// Reference to persistence futures returned when persisting snapshots.
#[derive(Debug, Clone)]
pub struct PersistenceReference {
    pub revision: String,
}

impl PersistenceReference {
    pub fn new(revision: String) -> Self {
        Self { revision }
    }
}
