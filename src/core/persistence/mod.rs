// siddhi_rust/src/core/persistence/mod.rs

pub mod data_source;
pub mod persistence_store; // For PersistenceStore traits
pub mod snapshot_service;

// Enhanced state management system (Phase 1)
pub mod state_holder;
pub mod state_registry;
pub mod state_manager;

// Incremental checkpointing system (Phase 2)
pub mod incremental;

pub use self::data_source::{DataSource, DataSourceConfig, SqliteDataSource};
pub use self::persistence_store::{
    FilePersistenceStore, InMemoryPersistenceStore, IncrementalPersistenceStore, PersistenceStore,
    SqlitePersistenceStore, RedisPersistenceStore,
};
pub use self::snapshot_service::SnapshotService;

// Enhanced state management exports
pub use self::state_holder::{
    StateHolder, StateSnapshot, StateError, StateSize, AccessPattern, 
    CheckpointId, ComponentId, SchemaVersion, ChangeLog, StateMetadata,
    SerializationHints, CompressionType
};
pub use self::state_registry::{
    StateRegistry, ComponentMetadata, ComponentPriority, ResourceRequirements,
    StateTopology, StateDependencyGraph
};
pub use self::state_manager::{
    UnifiedStateManager, StateConfig, CheckpointMode, CheckpointHandle,
    RecoveryStats, SchemaMigration, StateMetrics
};

// Incremental checkpointing exports
pub use self::incremental::{
    WriteAheadLog, CheckpointMerger, PersistenceBackend, RecoveryEngine, DistributedCoordinator,
    IncrementalCheckpoint, LogEntry, LogOffset, RecoveryPath, ClusterHealth, PartitionStatus,
    IncrementalCheckpointConfig, PersistenceBackendConfig, DistributedConfig
};
pub use self::incremental::write_ahead_log::WALConfig;
pub use self::incremental::checkpoint_merger::MergerConfig;
pub use self::incremental::recovery_engine::RecoveryConfig;
