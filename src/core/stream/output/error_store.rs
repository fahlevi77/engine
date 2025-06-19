use crate::core::exception::error::SiddhiError;
use std::sync::{Arc, Mutex};

pub trait ErrorStore: Send + Sync + std::fmt::Debug {
    fn store(&self, stream_id: &str, error: &SiddhiError);
}

#[derive(Default, Debug)]
pub struct InMemoryErrorStore {
    inner: Mutex<Vec<(String, SiddhiError)>>,
}

impl InMemoryErrorStore {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn errors(&self) -> Vec<(String, SiddhiError)> {
        self.inner.lock().unwrap().clone()
    }
}

impl ErrorStore for InMemoryErrorStore {
    fn store(&self, stream_id: &str, error: &SiddhiError) {
        self.inner
            .lock()
            .unwrap()
            .push((stream_id.to_string(), error.clone()));
    }
}
