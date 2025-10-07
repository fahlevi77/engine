// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::core::exception::error::EventFluxError;
use std::sync::Mutex;

pub trait ErrorStore: Send + Sync + std::fmt::Debug {
    fn store(&self, stream_id: &str, error: EventFluxError);
}

#[derive(Default, Debug)]
pub struct InMemoryErrorStore {
    inner: Mutex<Vec<(String, String)>>, // Store error messages as strings
}

impl InMemoryErrorStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn errors(&self) -> Vec<(String, String)> {
        self.inner.lock().unwrap().clone()
    }

    pub fn clear(&mut self) {
        self.inner.lock().unwrap().clear();
    }
}

impl ErrorStore for InMemoryErrorStore {
    fn store(&self, stream_id: &str, error: EventFluxError) {
        self.inner
            .lock()
            .unwrap()
            .push((stream_id.to_string(), error.to_string()));
    }
}
