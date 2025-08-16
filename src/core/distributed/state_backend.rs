// siddhi_rust/src/core/distributed/state_backend.rs

//! State Backend Abstraction
//! 
//! This module provides the state backend abstraction for distributed state management.
//! It supports multiple backends (Redis, Ignite, Hazelcast, etc.) for storing and
//! synchronizing state across nodes.

use async_trait::async_trait;
use super::{DistributedResult};
use std::sync::Arc;
use tokio::sync::RwLock;

/// State backend trait for distributed state management
#[async_trait]
pub trait StateBackend: Send + Sync {
    /// Initialize the state backend
    async fn initialize(&mut self) -> DistributedResult<()>;
    
    /// Get state value
    async fn get(&self, key: &str) -> DistributedResult<Option<Vec<u8>>>;
    
    /// Set state value
    async fn set(&self, key: &str, value: Vec<u8>) -> DistributedResult<()>;
    
    /// Delete state value
    async fn delete(&self, key: &str) -> DistributedResult<()>;
    
    /// Create a checkpoint
    async fn checkpoint(&self, checkpoint_id: &str) -> DistributedResult<()>;
    
    /// Restore from checkpoint
    async fn restore(&self, checkpoint_id: &str) -> DistributedResult<()>;
    
    /// Shutdown the backend
    async fn shutdown(&mut self) -> DistributedResult<()>;
}

/// In-memory state backend (for single-node)
pub struct InMemoryBackend {
    state: Arc<RwLock<std::collections::HashMap<String, Vec<u8>>>>,
}

impl InMemoryBackend {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait]
impl StateBackend for InMemoryBackend {
    async fn initialize(&mut self) -> DistributedResult<()> {
        Ok(())
    }
    
    async fn get(&self, key: &str) -> DistributedResult<Option<Vec<u8>>> {
        let state = self.state.read().await;
        Ok(state.get(key).cloned())
    }
    
    async fn set(&self, key: &str, value: Vec<u8>) -> DistributedResult<()> {
        let mut state = self.state.write().await;
        state.insert(key.to_string(), value);
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> DistributedResult<()> {
        let mut state = self.state.write().await;
        state.remove(key);
        Ok(())
    }
    
    async fn checkpoint(&self, _checkpoint_id: &str) -> DistributedResult<()> {
        Ok(())
    }
    
    async fn restore(&self, _checkpoint_id: &str) -> DistributedResult<()> {
        Ok(())
    }
    
    async fn shutdown(&mut self) -> DistributedResult<()> {
        let mut state = self.state.write().await;
        state.clear();
        Ok(())
    }
}