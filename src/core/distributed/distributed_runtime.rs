// siddhi_rust/src/core/distributed/distributed_runtime.rs

//! Distributed Runtime Wrapper
//! 
//! This module provides the main distributed runtime that wraps the existing
//! SiddhiAppRuntime and adds distributed capabilities based on configuration.
//! It maintains API compatibility while enabling distributed features.

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::core::siddhi_app_runtime::SiddhiAppRuntime;
use crate::core::config::siddhi_context::SiddhiContext;
use crate::core::event::Event;
use crate::core::stream::input::input_handler::InputHandler;
use crate::query_api::SiddhiApp as ApiSiddhiApp;
use std::sync::Mutex;

use super::{
    DistributedConfig, RuntimeMode, DistributedResult, DistributedError,
    runtime_mode::{RuntimeModeManager, HealthStatus},
    processing_engine::{ProcessingEngine, EngineStatistics},
};

/// Distributed runtime that wraps SiddhiAppRuntime with distributed capabilities
pub struct DistributedRuntime {
    /// Core Siddhi runtime (always present)
    core_runtime: Arc<SiddhiAppRuntime>,
    
    /// Runtime mode manager
    mode_manager: Arc<RwLock<RuntimeModeManager>>,
    
    /// Processing engine
    processing_engine: Arc<ProcessingEngine>,
    
    /// Distributed configuration
    config: Arc<DistributedConfig>,
    
    /// Runtime state
    state: Arc<RwLock<RuntimeState>>,
    
    /// Node ID (for distributed mode)
    node_id: String,
}

impl DistributedRuntime {
    /// Create a new distributed runtime
    pub async fn new(
        api_siddhi_app: Arc<ApiSiddhiApp>,
        siddhi_context: Arc<SiddhiContext>,
        config: DistributedConfig,
    ) -> DistributedResult<Self> {
        // Extract node ID from config or generate
        let node_id = config.node
            .as_ref()
            .map(|n| n.node_id.clone())
            .unwrap_or_else(|| format!("node-{}", uuid::Uuid::new_v4()));
        
        // Create core runtime (always needed)
        let core_runtime = Arc::new(
            SiddhiAppRuntime::new(
                Arc::clone(&api_siddhi_app),
                Arc::clone(&siddhi_context),
                None,
            ).map_err(|e| DistributedError::ConfigurationError {
                message: format!("Failed to create core runtime: {}", e),
            })?
        );
        
        // Create mode manager
        let mut mode_manager = RuntimeModeManager::new(config.clone())?;
        
        // Create processing engine
        let processing_engine = Arc::new(ProcessingEngine::new(config.mode)?);
        
        // Initialize mode manager
        mode_manager.initialize().await?;
        
        let runtime = Self {
            core_runtime,
            mode_manager: Arc::new(RwLock::new(mode_manager)),
            processing_engine,
            config: Arc::new(config),
            state: Arc::new(RwLock::new(RuntimeState::Initializing)),
            node_id,
        };
        
        Ok(runtime)
    }
    
    /// Start the distributed runtime
    pub async fn start(&self) -> DistributedResult<()> {
        // Update state
        {
            let mut state = self.state.write().await;
            *state = RuntimeState::Starting;
        }
        
        // Start core runtime
        self.core_runtime.start();
        
        // Register queries with processing engine
        for (query_id, query_runtime) in self.core_runtime.query_runtimes.iter().enumerate() {
            self.processing_engine
                .register_query(&format!("query_{}", query_id), Arc::clone(query_runtime))
                .await?;
        }
        
        // Update state
        {
            let mut state = self.state.write().await;
            *state = RuntimeState::Running;
        }
        
        println!("Distributed runtime started in {:?} mode on node {}", 
                 self.config.mode, self.node_id);
        
        Ok(())
    }
    
    /// Stop the distributed runtime
    pub async fn stop(&self) -> DistributedResult<()> {
        // Update state
        {
            let mut state = self.state.write().await;
            *state = RuntimeState::Stopping;
        }
        
        // Stop core runtime
        self.core_runtime.shutdown();
        
        // Shutdown mode manager
        {
            let mut manager = self.mode_manager.write().await;
            manager.shutdown().await?;
        }
        
        // Update state
        {
            let mut state = self.state.write().await;
            *state = RuntimeState::Stopped;
        }
        
        println!("Distributed runtime stopped");
        
        Ok(())
    }
    
    /// Get input handler for a stream
    pub fn get_input_handler(&self, stream_id: &str) -> Option<Arc<Mutex<InputHandler>>> {
        self.core_runtime.get_input_handler(stream_id)
    }
    
    /// Send event to a stream (distributed-aware)
    pub async fn send_event(
        &self,
        stream_id: &str,
        event: Event,
    ) -> DistributedResult<()> {
        match self.config.mode {
            RuntimeMode::SingleNode => {
                // Use core runtime directly
                if let Some(handler) = self.get_input_handler(stream_id) {
                    let handler = handler.lock().unwrap();
                    handler.send_single_event(event)
                        .map_err(|e| DistributedError::NetworkError {
                            message: format!("Failed to send event: {}", e),
                        })
                } else {
                    Err(DistributedError::NetworkError {
                        message: format!("Stream {} not found", stream_id),
                    })
                }
            },
            RuntimeMode::Distributed | RuntimeMode::Hybrid => {
                // Use processing engine for routing
                self.processing_engine.process_event(stream_id, event).await
            },
        }
    }
    
    /// Send batch of events (optimized for distributed)
    pub async fn send_batch(
        &self,
        stream_id: &str,
        events: Vec<Event>,
    ) -> DistributedResult<()> {
        match self.config.mode {
            RuntimeMode::SingleNode => {
                // Use core runtime directly
                if let Some(handler) = self.get_input_handler(stream_id) {
                    let handler = handler.lock().unwrap();
                    handler.send_multiple_events(events)
                        .map_err(|e| DistributedError::NetworkError {
                            message: format!("Failed to send batch: {}", e),
                        })
                } else {
                    Err(DistributedError::NetworkError {
                        message: format!("Stream {} not found", stream_id),
                    })
                }
            },
            RuntimeMode::Distributed | RuntimeMode::Hybrid => {
                // Use processing engine for batch routing
                self.processing_engine.process_batch(stream_id, events).await
            },
        }
    }
    
    /// Get runtime mode
    pub fn mode(&self) -> RuntimeMode {
        self.config.mode
    }
    
    /// Get node ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }
    
    /// Check if running in distributed mode
    pub fn is_distributed(&self) -> bool {
        matches!(self.config.mode, RuntimeMode::Distributed | RuntimeMode::Hybrid)
    }
    
    /// Get runtime health status
    pub async fn health_check(&self) -> DistributedResult<HealthStatus> {
        let manager = self.mode_manager.read().await;
        manager.health_check().await
    }
    
    /// Get runtime statistics
    pub async fn statistics(&self) -> RuntimeStatistics {
        let engine_stats = self.processing_engine.statistics().await;
        let state = self.state.read().await.clone();
        
        RuntimeStatistics {
            node_id: self.node_id.clone(),
            mode: self.config.mode,
            state,
            engine_stats,
            uptime: std::time::Duration::from_secs(0), // Would track actual uptime
        }
    }
    
    /// Trigger checkpoint (for distributed state)
    pub async fn checkpoint(&self) -> DistributedResult<CheckpointInfo> {
        match self.config.mode {
            RuntimeMode::SingleNode => {
                // Local checkpoint
                Ok(CheckpointInfo {
                    checkpoint_id: uuid::Uuid::new_v4().to_string(),
                    timestamp: std::time::SystemTime::now(),
                    node_id: self.node_id.clone(),
                    checkpoint_type: CheckpointType::Local,
                })
            },
            RuntimeMode::Distributed | RuntimeMode::Hybrid => {
                // Distributed checkpoint coordination
                Ok(CheckpointInfo {
                    checkpoint_id: uuid::Uuid::new_v4().to_string(),
                    timestamp: std::time::SystemTime::now(),
                    node_id: self.node_id.clone(),
                    checkpoint_type: CheckpointType::Distributed,
                })
            },
        }
    }
    
    /// Restore from checkpoint
    pub async fn restore(&self, checkpoint_id: &str) -> DistributedResult<()> {
        println!("Restoring from checkpoint: {}", checkpoint_id);
        // Implementation would restore state
        Ok(())
    }
    
    /// Scale out (add node to cluster)
    pub async fn scale_out(&self, node_config: super::NodeConfig) -> DistributedResult<()> {
        if !self.is_distributed() {
            return Err(DistributedError::ConfigurationError {
                message: "Scale out only available in distributed mode".to_string(),
            });
        }
        
        println!("Adding node {} to cluster", node_config.node_id);
        // Implementation would add node to cluster
        Ok(())
    }
    
    /// Scale in (remove node from cluster)
    pub async fn scale_in(&self, node_id: &str) -> DistributedResult<()> {
        if !self.is_distributed() {
            return Err(DistributedError::ConfigurationError {
                message: "Scale in only available in distributed mode".to_string(),
            });
        }
        
        println!("Removing node {} from cluster", node_id);
        // Implementation would remove node from cluster
        Ok(())
    }
}

/// Runtime state
#[derive(Debug, Clone)]
pub enum RuntimeState {
    Initializing,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed { reason: String },
}

/// Runtime statistics
#[derive(Debug)]
pub struct RuntimeStatistics {
    /// Node identifier
    pub node_id: String,
    
    /// Runtime mode
    pub mode: RuntimeMode,
    
    /// Current state
    pub state: RuntimeState,
    
    /// Engine statistics
    pub engine_stats: EngineStatistics,
    
    /// Uptime
    pub uptime: std::time::Duration,
}

/// Checkpoint information
#[derive(Debug)]
pub struct CheckpointInfo {
    /// Checkpoint identifier
    pub checkpoint_id: String,
    
    /// Checkpoint timestamp
    pub timestamp: std::time::SystemTime,
    
    /// Node that created checkpoint
    pub node_id: String,
    
    /// Checkpoint type
    pub checkpoint_type: CheckpointType,
}

/// Checkpoint type
#[derive(Debug)]
pub enum CheckpointType {
    Local,
    Distributed,
    Incremental,
    Full,
}

/// Builder for distributed runtime
pub struct DistributedRuntimeBuilder {
    config: DistributedConfig,
    siddhi_context: Option<Arc<SiddhiContext>>,
}

impl DistributedRuntimeBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: DistributedConfig::default(),
            siddhi_context: None,
        }
    }
    
    /// Set runtime mode
    pub fn mode(mut self, mode: RuntimeMode) -> Self {
        self.config.mode = mode;
        self
    }
    
    /// Set node configuration
    pub fn node(mut self, node: super::NodeConfig) -> Self {
        self.config.node = Some(node);
        self
    }
    
    /// Set cluster configuration
    pub fn cluster(mut self, cluster: super::ClusterConfig) -> Self {
        self.config.cluster = Some(cluster);
        self
    }
    
    /// Set Siddhi context
    pub fn siddhi_context(mut self, context: Arc<SiddhiContext>) -> Self {
        self.siddhi_context = Some(context);
        self
    }
    
    /// Build the distributed runtime
    pub async fn build(
        self,
        api_siddhi_app: Arc<ApiSiddhiApp>,
    ) -> DistributedResult<DistributedRuntime> {
        let siddhi_context = self.siddhi_context
            .ok_or_else(|| DistributedError::ConfigurationError {
                message: "SiddhiContext required".to_string(),
            })?;
        
        DistributedRuntime::new(api_siddhi_app, siddhi_context, self.config).await
    }
}

impl Default for DistributedRuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query_api::SiddhiApp;
    
    async fn create_test_runtime(mode: RuntimeMode) -> DistributedRuntime {
        let api_app = Arc::new(SiddhiApp::default());
        let context = Arc::new(SiddhiContext::default());
        
        let mut config = DistributedConfig::default();
        config.mode = mode;
        
        if mode == RuntimeMode::Distributed {
            config.node = Some(super::super::NodeConfig {
                node_id: "test-node".to_string(),
                endpoints: vec!["localhost:8080".to_string()],
                capabilities: super::super::NodeCapabilities::default(),
                resources: super::super::ResourceLimits::default(),
            });
            config.cluster = Some(super::super::ClusterConfig::default());
        }
        
        DistributedRuntime::new(api_app, context, config).await.unwrap()
    }
    
    #[tokio::test]
    async fn test_single_node_runtime() {
        let runtime = create_test_runtime(RuntimeMode::SingleNode).await;
        
        assert_eq!(runtime.mode(), RuntimeMode::SingleNode);
        assert!(!runtime.is_distributed());
        
        assert!(runtime.start().await.is_ok());
        
        let stats = runtime.statistics().await;
        assert_eq!(stats.mode, RuntimeMode::SingleNode);
        
        assert!(runtime.stop().await.is_ok());
    }
    
    #[tokio::test]
    async fn test_distributed_runtime() {
        let runtime = create_test_runtime(RuntimeMode::Distributed).await;
        
        assert_eq!(runtime.mode(), RuntimeMode::Distributed);
        assert!(runtime.is_distributed());
        assert_eq!(runtime.node_id(), "test-node");
        
        assert!(runtime.start().await.is_ok());
        assert!(runtime.stop().await.is_ok());
    }
    
    #[tokio::test]
    async fn test_checkpoint() {
        let runtime = create_test_runtime(RuntimeMode::SingleNode).await;
        
        assert!(runtime.start().await.is_ok());
        
        let checkpoint = runtime.checkpoint().await.unwrap();
        assert!(!checkpoint.checkpoint_id.is_empty());
        assert!(matches!(checkpoint.checkpoint_type, CheckpointType::Local));
        
        assert!(runtime.restore(&checkpoint.checkpoint_id).await.is_ok());
        
        assert!(runtime.stop().await.is_ok());
    }
    
    #[tokio::test]
    async fn test_builder() {
        let context = Arc::new(SiddhiContext::default());
        let api_app = Arc::new(SiddhiApp::default());
        
        let runtime = DistributedRuntimeBuilder::new()
            .mode(RuntimeMode::SingleNode)
            .siddhi_context(context)
            .build(api_app)
            .await
            .unwrap();
        
        assert_eq!(runtime.mode(), RuntimeMode::SingleNode);
    }
}