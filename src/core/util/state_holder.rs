use std::fmt::Debug;

/// Trait representing a stateful component which can snapshot and restore its state.
pub trait StateHolder: Send + Sync + Debug {
    /// Serialize the current state of the component into bytes.
    fn snapshot_state(&self) -> Vec<u8>;
    /// Restore the component state from the given bytes.
    fn restore_state(&self, snapshot: &[u8]);
}
