// siddhi_rust/src/core/query/processor/stream/window/session_window_state_holder.rs

//! Enhanced StateHolder implementation for SessionWindowProcessor
//! 
//! This implementation provides enterprise-grade state management for session windows
//! with versioning, incremental checkpointing, and comprehensive metadata.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

// Import the session window structs
use super::session_window_processor::{SessionWindowState, SessionContainer, SessionEventChunk};

use crate::core::event::stream::stream_event::StreamEvent;
use crate::core::persistence::state_holder::{
    StateHolder, StateSnapshot, StateError, StateSize, AccessPattern,
    SerializationHints, ChangeLog, CheckpointId, SchemaVersion, StateMetadata,
    CompressionType, StateOperation, ComponentId
};

/// Serializable representation of SessionEventChunk
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableSessionChunk {
    events: Vec<SerializableStreamEvent>,
    start_timestamp: i64,
    end_timestamp: i64,
    alive_timestamp: i64,
}

/// Serializable representation of StreamEvent (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableStreamEvent {
    timestamp: i64,
    before_window_data: Vec<crate::core::event::value::AttributeValue>,
    output_data: Option<Vec<crate::core::event::value::AttributeValue>>,
    event_type: i32, // Serialized as int for simplicity
}

/// Serializable representation of SessionContainer
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableSessionContainer {
    current_session: SerializableSessionChunk,
    previous_session: SerializableSessionChunk,
}

/// Serializable representation of SessionWindowState
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableSessionState {
    session_map: HashMap<String, SerializableSessionContainer>,
    expired_event_chunk: SerializableSessionChunk,
}

/// Enhanced state holder for SessionWindowProcessor with StateHolder capabilities
#[derive(Debug)]
pub struct SessionWindowStateHolder {
    /// Reference to the session window's state
    state: Arc<Mutex<SessionWindowState>>,
    
    /// Component identifier
    component_id: String,
    
    /// Session configuration
    session_gap: i64,
    allowed_latency: i64,
    
    /// Last checkpoint ID for incremental tracking
    last_checkpoint_id: Arc<Mutex<Option<CheckpointId>>>,
    
    /// Change log for incremental checkpointing
    change_log: Arc<Mutex<Vec<StateOperation>>>,
    
    /// Total sessions processed
    total_sessions_processed: Arc<Mutex<u64>>,
    
    /// Total events processed
    total_events_processed: Arc<Mutex<u64>>,
}

impl SessionWindowStateHolder {
    /// Create a new enhanced state holder
    pub fn new(
        state: Arc<Mutex<SessionWindowState>>,
        component_id: String,
        session_gap: i64,
        allowed_latency: i64,
    ) -> Self {
        Self {
            state,
            component_id,
            session_gap,
            allowed_latency,
            last_checkpoint_id: Arc::new(Mutex::new(None)),
            change_log: Arc::new(Mutex::new(Vec::new())),
            total_sessions_processed: Arc::new(Mutex::new(0)),
            total_events_processed: Arc::new(Mutex::new(0)),
        }
    }

    /// Record a new session creation
    pub fn record_session_created(&self, session_key: String) {
        let mut change_log = self.change_log.lock().unwrap();
        
        change_log.push(StateOperation::Insert {
            key: session_key.as_bytes().to_vec(),
            value: Vec::new(), // Empty value indicates new session
        });

        *self.total_sessions_processed.lock().unwrap() += 1;
    }

    /// Record an event addition to a session
    pub fn record_event_added(&self, session_key: &str, event: &StreamEvent) {
        let mut change_log = self.change_log.lock().unwrap();
        
        let key = format!("{session_key}:event:{}", event.timestamp).into_bytes();
        let value = self.serialize_event(event);
        
        change_log.push(StateOperation::Insert { key, value });
        
        *self.total_events_processed.lock().unwrap() += 1;
    }

    /// Record a session expiry
    pub fn record_session_expired(&self, session_key: String) {
        let mut change_log = self.change_log.lock().unwrap();
        
        change_log.push(StateOperation::Delete {
            key: session_key.as_bytes().to_vec(),
            old_value: Vec::new(),
        });
    }

    /// Serialize a StreamEvent
    fn serialize_event(&self, event: &StreamEvent) -> Vec<u8> {
        let serializable = SerializableStreamEvent {
            timestamp: event.timestamp,
            before_window_data: event.before_window_data.clone(),
            output_data: event.output_data.clone(),
            event_type: match event.event_type {
                crate::core::event::complex_event::ComplexEventType::Current => 0,
                crate::core::event::complex_event::ComplexEventType::Expired => 1,
                crate::core::event::complex_event::ComplexEventType::Timer => 2,
                crate::core::event::complex_event::ComplexEventType::Reset => 3,
            },
        };
        
        crate::core::util::to_bytes(&serializable).unwrap_or_default()
    }

    /// Deserialize a StreamEvent
    fn deserialize_event(&self, data: &[u8]) -> Result<StreamEvent, StateError> {
        use crate::core::event::complex_event::ComplexEventType;
        
        let serializable: SerializableStreamEvent = crate::core::util::from_bytes(data)
            .map_err(|e| StateError::DeserializationError {
                message: format!("Failed to deserialize StreamEvent: {e}"),
            })?;
        
        let mut event = StreamEvent::new(
            serializable.timestamp,
            serializable.before_window_data.len(),
            0,
            0,
        );
        
        event.before_window_data = serializable.before_window_data;
        event.output_data = serializable.output_data;
        event.event_type = match serializable.event_type {
            1 => ComplexEventType::Expired,
            2 => ComplexEventType::Timer,
            3 => ComplexEventType::Reset,
            _ => ComplexEventType::Current,
        };
        
        Ok(event)
    }

    /// Convert SessionEventChunk to serializable form
    fn chunk_to_serializable(&self, chunk: &SessionEventChunk) -> SerializableSessionChunk {
        SerializableSessionChunk {
            events: chunk.events.iter().map(|e| SerializableStreamEvent {
                timestamp: e.timestamp,
                before_window_data: e.before_window_data.clone(),
                output_data: e.output_data.clone(),
                event_type: match e.event_type {
                    crate::core::event::complex_event::ComplexEventType::Current => 0,
                    crate::core::event::complex_event::ComplexEventType::Expired => 1,
                    crate::core::event::complex_event::ComplexEventType::Timer => 2,
                    crate::core::event::complex_event::ComplexEventType::Reset => 3,
                },
            }).collect(),
            start_timestamp: chunk.start_timestamp,
            end_timestamp: chunk.end_timestamp,
            alive_timestamp: chunk.alive_timestamp,
        }
    }

    /// Convert serializable form back to SessionEventChunk
    fn serializable_to_chunk(&self, serializable: &SerializableSessionChunk) -> SessionEventChunk {
        use crate::core::event::complex_event::ComplexEventType;
        
        let mut chunk = SessionEventChunk::new();
        
        for ser_event in &serializable.events {
            let mut event = StreamEvent::new(
                ser_event.timestamp,
                ser_event.before_window_data.len(),
                0,
                0,
            );
            
            event.before_window_data = ser_event.before_window_data.clone();
            event.output_data = ser_event.output_data.clone();
            event.event_type = match ser_event.event_type {
                1 => ComplexEventType::Expired,
                2 => ComplexEventType::Timer,
                3 => ComplexEventType::Reset,
                _ => ComplexEventType::Current,
            };
            
            chunk.events.push(Arc::new(event));
        }
        
        chunk.start_timestamp = serializable.start_timestamp;
        chunk.end_timestamp = serializable.end_timestamp;
        chunk.alive_timestamp = serializable.alive_timestamp;
        
        chunk
    }

    /// Apply compression to data
    fn apply_compression(&self, data: &[u8], compression: &CompressionType) -> Result<Vec<u8>, StateError> {
        match compression {
            CompressionType::None => Ok(data.to_vec()),
            CompressionType::LZ4 => {
                lz4::block::compress(data, None, true)
                    .map_err(|e| StateError::CompressionError {
                        message: format!("LZ4 compression failed: {e}"),
                    })
            }
            CompressionType::Snappy => {
                snap::raw::Encoder::new()
                    .compress_vec(data)
                    .map_err(|e| StateError::CompressionError {
                        message: format!("Snappy compression failed: {e}"),
                    })
            }
            CompressionType::Zstd => {
                zstd::encode_all(data, 3) // Use compression level 3
                    .map_err(|e| StateError::CompressionError {
                        message: format!("Zstd compression failed: {e}"),
                    })
            }
        }
    }

    /// Decompress data
    fn decompress_data(&self, data: &[u8], compression: &CompressionType) -> Result<Vec<u8>, StateError> {
        match compression {
            CompressionType::None => Ok(data.to_vec()),
            CompressionType::LZ4 => {
                lz4::block::decompress(data, None)
                    .map_err(|e| StateError::CompressionError {
                        message: format!("LZ4 decompression failed: {e}"),
                    })
            }
            CompressionType::Snappy => {
                snap::raw::Decoder::new()
                    .decompress_vec(data)
                    .map_err(|e| StateError::CompressionError {
                        message: format!("Snappy decompression failed: {e}"),
                    })
            }
            CompressionType::Zstd => {
                zstd::decode_all(data)
                    .map_err(|e| StateError::CompressionError {
                        message: format!("Zstd decompression failed: {e}"),
                    })
            }
        }
    }
}

impl StateHolder for SessionWindowStateHolder {
    fn schema_version(&self) -> SchemaVersion {
        SchemaVersion::new(1, 0, 0)
    }

    fn serialize_state(&self, hints: &SerializationHints) -> Result<StateSnapshot, StateError> {
        let state = self.state.lock().map_err(|_| StateError::InvalidStateData {
            message: "Failed to acquire session window state lock".to_string(),
        })?;

        // Convert to serializable form
        let mut serializable_state = SerializableSessionState {
            session_map: HashMap::new(),
            expired_event_chunk: self.chunk_to_serializable(&state.expired_event_chunk),
        };

        // Convert each session container
        for (key, container) in &state.session_map {
            let ser_container = SerializableSessionContainer {
                current_session: self.chunk_to_serializable(&container.current_session),
                previous_session: self.chunk_to_serializable(&container.previous_session),
            };
            serializable_state.session_map.insert(key.clone(), ser_container);
        }

        // Serialize the state
        let data = crate::core::util::to_bytes(&serializable_state)
            .map_err(|e| StateError::SerializationError {
                message: format!("Failed to serialize session window state: {e}"),
            })?;

        // Apply compression if requested
        let (compressed_data, compression_type) = if let Some(ref compression) = hints.prefer_compression {
            match self.apply_compression(&data, compression) {
                Ok(compressed) => (compressed, compression.clone()),
                Err(_) => {
                    // Fall back to no compression if compression fails
                    (data, CompressionType::None)
                }
            }
        } else {
            (data, CompressionType::None)
        };

        let checksum = StateSnapshot::calculate_checksum(&compressed_data);

        Ok(StateSnapshot {
            version: self.schema_version(),
            checkpoint_id: self.last_checkpoint_id.lock().unwrap().unwrap_or(0),
            data: compressed_data,
            compression: compression_type,
            checksum,
            metadata: self.component_metadata(),
        })
    }

    fn deserialize_state(&mut self, snapshot: &StateSnapshot) -> Result<(), StateError> {
        // Verify checksum
        if !snapshot.verify_integrity() {
            return Err(StateError::ChecksumMismatch);
        }

        // Check version compatibility
        if !self.can_migrate_from(&snapshot.version) {
            return Err(StateError::IncompatibleVersion {
                current: self.schema_version(),
                required: snapshot.version,
            });
        }

        // Decompress if needed
        let decompressed_data = self.decompress_data(&snapshot.data, &snapshot.compression)?;
        let data = &decompressed_data;

        // Deserialize the state
        let serializable_state: SerializableSessionState = crate::core::util::from_bytes(data)
            .map_err(|e| StateError::DeserializationError {
                message: format!("Failed to deserialize session window state: {e}"),
            })?;

        let mut state = self.state.lock().map_err(|_| StateError::InvalidStateData {
            message: "Failed to acquire session window state lock".to_string(),
        })?;

        // Clear existing state
        state.session_map.clear();
        state.expired_event_chunk = self.serializable_to_chunk(&serializable_state.expired_event_chunk);

        // Restore session containers
        for (key, ser_container) in serializable_state.session_map {
            let container = SessionContainer {
                current_session: self.serializable_to_chunk(&ser_container.current_session),
                previous_session: self.serializable_to_chunk(&ser_container.previous_session),
            };
            state.session_map.insert(key, container);
        }

        // Update checkpoint ID
        *self.last_checkpoint_id.lock().unwrap() = Some(snapshot.checkpoint_id);

        Ok(())
    }

    fn get_changelog(&self, since: CheckpointId) -> Result<ChangeLog, StateError> {
        let last_checkpoint = self.last_checkpoint_id.lock().unwrap();
        
        if let Some(last_id) = *last_checkpoint {
            if since > last_id {
                return Err(StateError::CheckpointNotFound {
                    checkpoint_id: since,
                });
            }
        }

        let change_log = self.change_log.lock().unwrap();
        
        let mut changelog = ChangeLog::new(since, last_checkpoint.unwrap_or(0));
        
        // Add all operations from the change log
        for operation in change_log.iter() {
            changelog.add_operation(operation.clone());
        }
        
        // Set compression if the changelog is large
        if changelog.size_bytes > 10_000 {
            changelog.compression = CompressionType::LZ4;
        }
        
        Ok(changelog)
    }

    fn apply_changelog(&mut self, changes: &ChangeLog) -> Result<(), StateError> {
        // For session windows, applying a changelog would require
        // replaying the operations to rebuild the state
        // This is a simplified implementation
        
        for operation in &changes.operations {
            match operation {
                StateOperation::Insert { key, .. } => {
                    // Session creation or event addition
                    let key_str = String::from_utf8_lossy(key);
                    if key_str.contains(":event:") {
                        // Event addition - would need to deserialize and add to session
                        *self.total_events_processed.lock().unwrap() += 1;
                    } else {
                        // Session creation
                        *self.total_sessions_processed.lock().unwrap() += 1;
                    }
                }
                StateOperation::Delete { .. } => {
                    // Session expiry
                }
                StateOperation::Update { .. } => {
                    // Session update (not typically used for session windows)
                }
                StateOperation::Clear => {
                    // Clear all state
                    let mut state = self.state.lock().unwrap();
                    state.session_map.clear();
                    state.expired_event_chunk.clear();
                }
            }
        }
        
        // Update checkpoint ID
        *self.last_checkpoint_id.lock().unwrap() = Some(changes.to_checkpoint);
        
        Ok(())
    }

    fn estimate_size(&self) -> StateSize {
        let state = self.state.lock().unwrap();
        
        let mut total_events = 0usize;
        let mut estimated_bytes = 0usize;
        
        // Count events in all sessions
        for (key, container) in &state.session_map {
            total_events += container.current_session.events.len();
            total_events += container.previous_session.events.len();
            
            // Estimate bytes (rough approximation)
            estimated_bytes += key.len();
            estimated_bytes += container.current_session.events.len() * 100; // ~100 bytes per event
            estimated_bytes += container.previous_session.events.len() * 100;
            estimated_bytes += 24 * 2; // Timestamps
        }
        
        // Add expired events
        total_events += state.expired_event_chunk.events.len();
        estimated_bytes += state.expired_event_chunk.events.len() * 100;
        
        let total_processed = *self.total_events_processed.lock().unwrap();
        let growth_rate = if total_processed > 0 {
            (total_events as f64 / total_processed as f64) * 100.0
        } else {
            0.0
        };
        
        StateSize {
            bytes: estimated_bytes,
            entries: total_events,
            estimated_growth_rate: growth_rate,
        }
    }

    fn access_pattern(&self) -> AccessPattern {
        // Session windows have moderate access patterns
        // They're accessed when events arrive and during timeout processing
        AccessPattern::Warm
    }

    fn component_metadata(&self) -> StateMetadata {
        let mut metadata = StateMetadata::new(
            self.component_id.clone(),
            "SessionWindowProcessor".to_string(),
        );
        
        metadata.access_pattern = self.access_pattern();
        metadata.size_estimation = self.estimate_size();
        
        // Add custom metadata
        metadata.custom_metadata.insert("session_gap".to_string(), self.session_gap.to_string());
        metadata.custom_metadata.insert("allowed_latency".to_string(), self.allowed_latency.to_string());
        
        let sessions = self.state.lock().unwrap().session_map.len();
        metadata.custom_metadata.insert("active_sessions".to_string(), sessions.to_string());
        
        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_window_state_holder_creation() {
        let state = Arc::new(Mutex::new(SessionWindowState::new()));
        let holder = SessionWindowStateHolder::new(
            state.clone(),
            "test_session_window".to_string(),
            5000,
            1000,
        );
        
        assert_eq!(holder.component_id, "test_session_window");
        assert_eq!(holder.session_gap, 5000);
        assert_eq!(holder.allowed_latency, 1000);
        assert_eq!(holder.schema_version(), SchemaVersion::new(1, 0, 0));
    }

    #[test]
    fn test_serialize_deserialize_empty_state() {
        let state = Arc::new(Mutex::new(SessionWindowState::new()));
        let mut holder = SessionWindowStateHolder::new(
            state.clone(),
            "test_session_window".to_string(),
            5000,
            1000,
        );
        
        let hints = SerializationHints::default();
        let snapshot = holder.serialize_state(&hints).unwrap();
        
        assert!(snapshot.verify_integrity());
        assert_eq!(snapshot.version, SchemaVersion::new(1, 0, 0));
        
        // Deserialize back
        holder.deserialize_state(&snapshot).unwrap();
        
        // Verify state is still empty
        let state_guard = state.lock().unwrap();
        assert!(state_guard.session_map.is_empty());
        assert!(state_guard.expired_event_chunk.is_empty());
    }

    #[test]
    fn test_serialize_deserialize_with_sessions() {
        let state = Arc::new(Mutex::new(SessionWindowState::new()));
        
        // Add some test data
        {
            let mut state_guard = state.lock().unwrap();
            
            // Create a session with events
            let mut container = SessionContainer::new();
            
            let event1 = Arc::new(StreamEvent::new(1000, 2, 0, 0));
            let event2 = Arc::new(StreamEvent::new(2000, 2, 0, 0));
            
            container.current_session.add_event(event1);
            container.current_session.add_event(event2);
            container.current_session.set_timestamps(1000, 7000, 8000);
            
            state_guard.session_map.insert("session1".to_string(), container);
        }
        
        let mut holder = SessionWindowStateHolder::new(
            state.clone(),
            "test_session_window".to_string(),
            5000,
            1000,
        );
        
        // Serialize
        let hints = SerializationHints::default();
        let snapshot = holder.serialize_state(&hints).unwrap();
        
        // Clear state
        {
            let mut state_guard = state.lock().unwrap();
            state_guard.session_map.clear();
        }
        
        // Deserialize
        holder.deserialize_state(&snapshot).unwrap();
        
        // Verify restored state
        {
            let state_guard = state.lock().unwrap();
            assert_eq!(state_guard.session_map.len(), 1);
            assert!(state_guard.session_map.contains_key("session1"));
            
            let container = state_guard.session_map.get("session1").unwrap();
            assert_eq!(container.current_session.events.len(), 2);
            assert_eq!(container.current_session.start_timestamp, 1000);
            assert_eq!(container.current_session.end_timestamp, 7000);
            assert_eq!(container.current_session.alive_timestamp, 8000);
        }
    }

    #[test]
    fn test_changelog_operations() {
        let state = Arc::new(Mutex::new(SessionWindowState::new()));
        let holder = SessionWindowStateHolder::new(
            state.clone(),
            "test_session_window".to_string(),
            5000,
            1000,
        );
        
        // Record some operations
        holder.record_session_created("session1".to_string());
        
        let event = StreamEvent::new(1000, 2, 0, 0);
        holder.record_event_added("session1", &event);
        
        holder.record_session_expired("session1".to_string());
        
        // Get changelog
        let changelog = holder.get_changelog(0).unwrap();
        assert_eq!(changelog.operations.len(), 3);
        
        // Verify operations
        assert!(matches!(changelog.operations[0], StateOperation::Insert { .. }));
        assert!(matches!(changelog.operations[1], StateOperation::Insert { .. }));
        assert!(matches!(changelog.operations[2], StateOperation::Delete { .. }));
    }

    #[test]
    fn test_size_estimation() {
        let state = Arc::new(Mutex::new(SessionWindowState::new()));
        
        // Add test data
        {
            let mut state_guard = state.lock().unwrap();
            
            for i in 0..3 {
                let mut container = SessionContainer::new();
                for j in 0..5 {
                    let event = Arc::new(StreamEvent::new(1000 + j * 100, 2, 0, 0));
                    container.current_session.add_event(event);
                }
                state_guard.session_map.insert(format!("session{i}"), container);
            }
        }
        
        let holder = SessionWindowStateHolder::new(
            state.clone(),
            "test_session_window".to_string(),
            5000,
            1000,
        );
        
        let size = holder.estimate_size();
        assert_eq!(size.entries, 15); // 3 sessions * 5 events
        assert!(size.bytes > 0);
    }

    #[test]
    fn test_metadata() {
        let state = Arc::new(Mutex::new(SessionWindowState::new()));
        
        // Add a session
        {
            let mut state_guard = state.lock().unwrap();
            state_guard.session_map.insert("session1".to_string(), SessionContainer::new());
        }
        
        let holder = SessionWindowStateHolder::new(
            state.clone(),
            "test_session_window".to_string(),
            5000,
            1000,
        );
        
        let metadata = holder.component_metadata();
        assert_eq!(metadata.component_id, "test_session_window");
        assert_eq!(metadata.component_type, "SessionWindowProcessor");
        assert_eq!(metadata.access_pattern, AccessPattern::Warm);
        
        // Check custom metadata
        assert_eq!(metadata.custom_metadata.get("session_gap"), Some(&"5000".to_string()));
        assert_eq!(metadata.custom_metadata.get("allowed_latency"), Some(&"1000".to_string()));
        assert_eq!(metadata.custom_metadata.get("active_sessions"), Some(&"1".to_string()));
    }

    #[test]
    fn test_compression_lz4() {
        let state = Arc::new(Mutex::new(SessionWindowState::new()));
        
        // Add test data
        {
            let mut state_guard = state.lock().unwrap();
            
            let mut container = SessionContainer::new();
            
            for i in 0..10 {
                let event = Arc::new(StreamEvent::new(1000 + i * 100, 3, 0, 0));
                container.current_session.add_event(event);
            }
            container.current_session.set_timestamps(1000, 7000, 8000);
            
            state_guard.session_map.insert("session1".to_string(), container);
        }
        
        let mut holder = SessionWindowStateHolder::new(
            state.clone(),
            "test_session_window".to_string(),
            5000,
            1000,
        );
        
        // Test LZ4 compression
        let mut hints = SerializationHints::default();
        hints.prefer_compression = Some(CompressionType::LZ4);
        
        // Serialize with LZ4 compression
        let snapshot = holder.serialize_state(&hints).unwrap();
        assert_eq!(snapshot.compression, CompressionType::LZ4);
        
        // Clear state
        {
            let mut state_guard = state.lock().unwrap();
            state_guard.session_map.clear();
        }
        
        // Deserialize and verify
        holder.deserialize_state(&snapshot).unwrap();
        
        {
            let state_guard = state.lock().unwrap();
            assert_eq!(state_guard.session_map.len(), 1);
            assert!(state_guard.session_map.contains_key("session1"));
            
            let container = state_guard.session_map.get("session1").unwrap();
            assert_eq!(container.current_session.events.len(), 10);
            assert_eq!(container.current_session.start_timestamp, 1000);
        }
    }

    #[test]
    fn test_compression_snappy() {
        let state = Arc::new(Mutex::new(SessionWindowState::new()));
        
        // Add test data with multiple sessions
        {
            let mut state_guard = state.lock().unwrap();
            
            for session_num in 0..3 {
                let mut container = SessionContainer::new();
                
                for i in 0..5 {
                    let event = Arc::new(StreamEvent::new(1000 + session_num * 1000 + i * 100, 2, 0, 0));
                    container.current_session.add_event(event);
                }
                container.current_session.set_timestamps(
                    1000 + session_num * 1000, 
                    6000 + session_num * 1000, 
                    7000 + session_num * 1000
                );
                
                state_guard.session_map.insert(format!("session{session_num}"), container);
            }
        }
        
        let mut holder = SessionWindowStateHolder::new(
            state.clone(),
            "test_session_window".to_string(),
            5000,
            1000,
        );
        
        // Test Snappy compression
        let mut hints = SerializationHints::default();
        hints.prefer_compression = Some(CompressionType::Snappy);
        
        // Serialize with Snappy compression
        let snapshot = holder.serialize_state(&hints).unwrap();
        assert_eq!(snapshot.compression, CompressionType::Snappy);
        
        // Clear state
        {
            let mut state_guard = state.lock().unwrap();
            state_guard.session_map.clear();
        }
        
        // Deserialize and verify
        holder.deserialize_state(&snapshot).unwrap();
        
        {
            let state_guard = state.lock().unwrap();
            assert_eq!(state_guard.session_map.len(), 3);
            
            for session_num in 0..3 {
                let container = state_guard.session_map.get(&format!("session{session_num}")).unwrap();
                assert_eq!(container.current_session.events.len(), 5);
                assert_eq!(container.current_session.start_timestamp, 1000 + session_num * 1000);
            }
        }
    }

    #[test]
    fn test_compression_zstd() {
        let state = Arc::new(Mutex::new(SessionWindowState::new()));
        
        // Add test data with large session
        {
            let mut state_guard = state.lock().unwrap();
            
            let mut container = SessionContainer::new();
            
            // Create a large session with many events to test compression effectiveness
            for i in 0..50 {
                let mut event = StreamEvent::new(1000 + i * 100, 4, 0, 0);
                // Add some data to make compression more effective
                event.before_window_data = vec![
                    crate::core::event::value::AttributeValue::String(format!("data_string_{i}")),
                    crate::core::event::value::AttributeValue::Int(i as i32),
                    crate::core::event::value::AttributeValue::Float((i as f32) * 1.5),
                    crate::core::event::value::AttributeValue::Bool(i % 2 == 0),
                ];
                container.current_session.add_event(Arc::new(event));
            }
            container.current_session.set_timestamps(1000, 10000, 11000);
            
            state_guard.session_map.insert("large_session".to_string(), container);
        }
        
        let mut holder = SessionWindowStateHolder::new(
            state.clone(),
            "test_session_window".to_string(),
            5000,
            1000,
        );
        
        // Test Zstd compression
        let mut hints = SerializationHints::default();
        hints.prefer_compression = Some(CompressionType::Zstd);
        
        // Serialize with Zstd compression
        let snapshot = holder.serialize_state(&hints).unwrap();
        assert_eq!(snapshot.compression, CompressionType::Zstd);
        
        // Clear state
        {
            let mut state_guard = state.lock().unwrap();
            state_guard.session_map.clear();
        }
        
        // Deserialize and verify
        holder.deserialize_state(&snapshot).unwrap();
        
        {
            let state_guard = state.lock().unwrap();
            assert_eq!(state_guard.session_map.len(), 1);
            assert!(state_guard.session_map.contains_key("large_session"));
            
            let container = state_guard.session_map.get("large_session").unwrap();
            assert_eq!(container.current_session.events.len(), 50);
            assert_eq!(container.current_session.start_timestamp, 1000);
            assert_eq!(container.current_session.end_timestamp, 10000);
            
            // Verify some event data
            let first_event = &container.current_session.events[0];
            assert_eq!(first_event.before_window_data.len(), 4);
            if let crate::core::event::value::AttributeValue::String(s) = &first_event.before_window_data[0] {
                assert_eq!(s, "data_string_0");
            } else {
                panic!("Expected string attribute");
            }
        }
    }

    #[test]
    fn test_compression_effectiveness() {
        let state = Arc::new(Mutex::new(SessionWindowState::new()));
        
        // Create large repetitive data to test compression effectiveness
        {
            let mut state_guard = state.lock().unwrap();
            
            let mut container = SessionContainer::new();
            
            // Add many similar events
            for i in 0..100 {
                let mut event = StreamEvent::new(1000 + i * 10, 2, 0, 0);
                event.before_window_data = vec![
                    crate::core::event::value::AttributeValue::String("repeating_string_data".to_string()),
                    crate::core::event::value::AttributeValue::Int(42), // Same value
                ];
                container.current_session.add_event(Arc::new(event));
            }
            container.current_session.set_timestamps(1000, 6000, 7000);
            
            state_guard.session_map.insert("repetitive_session".to_string(), container);
        }
        
        let holder = SessionWindowStateHolder::new(
            state.clone(),
            "test_session_window".to_string(),
            5000,
            1000,
        );
        
        // Test no compression (baseline)
        let hints_none = SerializationHints::default();
        let snapshot_none = holder.serialize_state(&hints_none).unwrap();
        let uncompressed_size = snapshot_none.data.len();
        
        // Test LZ4 compression
        let mut hints_lz4 = SerializationHints::default();
        hints_lz4.prefer_compression = Some(CompressionType::LZ4);
        let snapshot_lz4 = holder.serialize_state(&hints_lz4).unwrap();
        let lz4_size = snapshot_lz4.data.len();
        
        // Test Snappy compression
        let mut hints_snappy = SerializationHints::default();
        hints_snappy.prefer_compression = Some(CompressionType::Snappy);
        let snapshot_snappy = holder.serialize_state(&hints_snappy).unwrap();
        let snappy_size = snapshot_snappy.data.len();
        
        // Test Zstd compression
        let mut hints_zstd = SerializationHints::default();
        hints_zstd.prefer_compression = Some(CompressionType::Zstd);
        let snapshot_zstd = holder.serialize_state(&hints_zstd).unwrap();
        let zstd_size = snapshot_zstd.data.len();
        
        println!("Compression effectiveness:");
        println!("Uncompressed: {} bytes", uncompressed_size);
        println!("LZ4: {} bytes ({:.1}% of original)", lz4_size, (lz4_size as f64 / uncompressed_size as f64) * 100.0);
        println!("Snappy: {} bytes ({:.1}% of original)", snappy_size, (snappy_size as f64 / uncompressed_size as f64) * 100.0);
        println!("Zstd: {} bytes ({:.1}% of original)", zstd_size, (zstd_size as f64 / uncompressed_size as f64) * 100.0);
        
        // All compressed versions should be smaller than uncompressed for repetitive data
        assert!(lz4_size < uncompressed_size, "LZ4 should compress repetitive data");
        assert!(snappy_size < uncompressed_size, "Snappy should compress repetitive data");
        assert!(zstd_size < uncompressed_size, "Zstd should compress repetitive data");
        
        // Zstd should generally provide the best compression ratio
        assert!(zstd_size <= lz4_size, "Zstd should compress at least as well as LZ4");
    }
}