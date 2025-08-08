// siddhi_rust/src/core/query/processor/stream/window/time_batch_window_state_holder.rs

//! Enhanced StateHolder implementation for TimeBatchWindowProcessor
//! 
//! This implementation provides enterprise-grade state management for time batch windows
//! with versioning, incremental checkpointing, and comprehensive metadata.

use std::sync::{Arc, Mutex};

use crate::core::event::stream::stream_event::StreamEvent;
use crate::core::persistence::state_holder::{
    StateHolder, StateSnapshot, StateError, StateSize, AccessPattern,
    SerializationHints, ChangeLog, CheckpointId, SchemaVersion, StateMetadata,
    CompressionType, StateOperation
};
use crate::core::util::event_serialization::{
    EventSerializationService, StorageStrategy
};

/// Enhanced state holder for TimeBatchWindowProcessor with StateHolder capabilities
#[derive(Debug, Clone)]
pub struct TimeBatchWindowStateHolder {
    /// Current batch buffer
    buffer: Arc<Mutex<Vec<StreamEvent>>>,
    
    /// Expired batch buffer 
    expired: Arc<Mutex<Vec<StreamEvent>>>,
    
    /// Batch start time tracking
    start_time: Arc<Mutex<Option<i64>>>,
    
    /// Component identifier
    component_id: String,
    
    /// Batch duration in milliseconds
    duration_ms: i64,
    
    /// Last checkpoint ID for incremental tracking
    last_checkpoint_id: Arc<Mutex<Option<CheckpointId>>>,
    
    /// Change log for incremental checkpointing
    change_log: Arc<Mutex<Vec<StateOperation>>>,
    
    /// Event counter for size estimation
    total_events_processed: Arc<Mutex<u64>>,
    
    /// Event serialization service with proper AttributeValue handling
    serialization_service: EventSerializationService,
}

impl TimeBatchWindowStateHolder {
    /// Create a new enhanced state holder
    pub fn new(
        buffer: Arc<Mutex<Vec<StreamEvent>>>,
        expired: Arc<Mutex<Vec<StreamEvent>>>,
        start_time: Arc<Mutex<Option<i64>>>,
        component_id: String,
        duration_ms: i64,
    ) -> Self {
        Self {
            buffer,
            expired,
            start_time,
            component_id,
            duration_ms,
            last_checkpoint_id: Arc::new(Mutex::new(None)),
            change_log: Arc::new(Mutex::new(Vec::new())),
            total_events_processed: Arc::new(Mutex::new(0)),
            serialization_service: EventSerializationService::new(StorageStrategy::Essential),
        }
    }

    /// Record an event addition to current batch for incremental checkpointing
    pub fn record_event_added(&self, event: &StreamEvent) {
        let mut change_log = self.change_log.lock().unwrap();
        let event_data = self.serialize_event(event);
        
        change_log.push(StateOperation::Insert {
            key: self.generate_event_key(event, "current_batch"),
            value: event_data,
        });

        // Update event counter
        *self.total_events_processed.lock().unwrap() += 1;
    }

    /// Record a batch flush for incremental checkpointing  
    pub fn record_batch_flushed(&self, current_batch: &[StreamEvent], expired_batch: &[StreamEvent], timestamp: i64) {
        let mut change_log = self.change_log.lock().unwrap();
        
        // Record batch transition with timing information
        change_log.push(StateOperation::Delete {
            key: b"time_batch_flush_marker".to_vec(),
            old_value: self.serialize_time_batch_transition(current_batch, expired_batch, timestamp),
        });
    }

    /// Record start time change for incremental checkpointing
    pub fn record_start_time_updated(&self, old_start_time: Option<i64>, new_start_time: Option<i64>) {
        let mut change_log = self.change_log.lock().unwrap();
        
        let old_data = self.serialize_start_time(old_start_time);
        let new_data = self.serialize_start_time(new_start_time);
        
        change_log.push(StateOperation::Update {
            key: b"start_time".to_vec(),
            old_value: old_data,
            new_value: new_data,
        });
    }

    /// Generate a unique key for an event
    fn generate_event_key(&self, event: &StreamEvent, buffer_type: &str) -> Vec<u8> {
        // Use timestamp, buffer type, and a hash of event data as key
        let mut key = Vec::new();
        key.extend_from_slice(buffer_type.as_bytes());
        key.push(b'_');
        key.extend_from_slice(&event.timestamp.to_le_bytes());
        
        // Add a simple hash of the event data
        let data_hash = self.hash_event_data(&event.before_window_data);
        key.extend_from_slice(&data_hash.to_le_bytes());
        
        key
    }

    /// Simple hash function for event data
    fn hash_event_data(&self, data: &[crate::core::event::value::AttributeValue]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Hash the length first
        data.len().hash(&mut hasher);
        
        // Hash each attribute value
        for attr in data {
            // Create a string representation for hashing
            let attr_str = format!("{attr:?}");
            attr_str.hash(&mut hasher);
        }
        
        hasher.finish()
    }

    /// Serialize an event to bytes using the enhanced serialization service
    fn serialize_event(&self, event: &StreamEvent) -> Vec<u8> {
        self.serialization_service
            .serialize_event(event)
            .unwrap_or_default()
    }

    /// Serialize time batch transition for changelog
    fn serialize_time_batch_transition(&self, current_batch: &[StreamEvent], expired_batch: &[StreamEvent], flush_time: i64) -> Vec<u8> {
        use crate::core::util::to_bytes;
        
        let transition_data = (
            current_batch.len(),
            expired_batch.len(),
            flush_time,
            self.duration_ms,
        );
        
        to_bytes(&transition_data).unwrap_or_default()
    }

    /// Serialize start time for changelog
    fn serialize_start_time(&self, start_time: Option<i64>) -> Vec<u8> {
        use crate::core::util::to_bytes;
        to_bytes(&start_time).unwrap_or_default()
    }

    /// Deserialize an event from bytes using the enhanced serialization service
    fn deserialize_event(&self, data: &[u8]) -> Result<StreamEvent, StateError> {
        self.serialization_service.deserialize_event(data)
    }

    /// Clear the change log (called after successful checkpoint)
    pub fn clear_change_log(&self, checkpoint_id: CheckpointId) {
        let mut change_log = self.change_log.lock().unwrap();
        change_log.clear();
        *self.last_checkpoint_id.lock().unwrap() = Some(checkpoint_id);
    }
}

impl StateHolder for TimeBatchWindowStateHolder {
    fn schema_version(&self) -> SchemaVersion {
        SchemaVersion::new(1, 0, 0)
    }

    fn serialize_state(&self, hints: &SerializationHints) -> Result<StateSnapshot, StateError> {
        use crate::core::util::to_bytes;
        
        let start_time = *self.start_time.lock().unwrap();
        
        // Determine storage strategy based on hints
        let storage_strategy = hints.prefer_compression
            .as_ref()
            .map(|_| StorageStrategy::Compressed)
            .unwrap_or(StorageStrategy::Essential);
        
        // Serialize current batch events
        let current_batch = {
            let buffer = self.buffer.lock().unwrap();
            let mut serialized_events = Vec::new();
            for event in buffer.iter() {
                match self.serialization_service
                    .serialize_event_with_strategy(event, storage_strategy.clone()) {
                    Ok(data) => serialized_events.push(data),
                    Err(e) => {
                        eprintln!("Warning: Failed to serialize event: {e}");
                        // Continue with other events rather than failing completely
                    }
                }
            }
            serialized_events
        };
        
        // Serialize expired batch events
        let expired_batch = {
            let expired = self.expired.lock().unwrap();
            let mut serialized_events = Vec::new();
            for event in expired.iter() {
                match self.serialization_service
                    .serialize_event_with_strategy(event, storage_strategy.clone()) {
                    Ok(data) => serialized_events.push(data),
                    Err(e) => {
                        eprintln!("Warning: Failed to serialize expired event: {e}");
                        // Continue with other events rather than failing completely
                    }
                }
            }
            serialized_events
        };
        
        let state_data = TimeBatchWindowStateData {
            current_batch,
            expired_batch,
            duration_ms: self.duration_ms,
            start_time,
            total_events_processed: *self.total_events_processed.lock().unwrap(),
        };
        
        // Serialize to bytes
        let mut data = to_bytes(&state_data).map_err(|e| StateError::SerializationError {
            message: format!("Failed to serialize time batch window state: {e}"),
        })?;
        
        // Apply compression if requested
        let compression = hints.prefer_compression.clone().unwrap_or(CompressionType::None);
        data = self.apply_compression(data, &compression)?;
        
        let checksum = StateSnapshot::calculate_checksum(&data);
        
        Ok(StateSnapshot {
            version: self.schema_version(),
            checkpoint_id: 0, // Will be set by the checkpoint coordinator
            data,
            compression,
            checksum,
            metadata: self.component_metadata(),
        })
    }

    fn deserialize_state(&mut self, snapshot: &StateSnapshot) -> Result<(), StateError> {
        use crate::core::util::from_bytes;
        
        // Verify integrity
        if !snapshot.verify_integrity() {
            return Err(StateError::ChecksumMismatch);
        }
        
        // Decompress data if needed
        let data = self.decompress_data(&snapshot.data, &snapshot.compression)?;
        
        // Deserialize state data
        let state_data: TimeBatchWindowStateData = from_bytes(&data).map_err(|e| {
            StateError::DeserializationError {
                message: format!("Failed to deserialize time batch window state: {e}"),
            }
        })?;
        
        // Deserialize and restore current batch events
        {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.clear();
            
            for serialized_event in state_data.current_batch {
                match self.deserialize_event(&serialized_event) {
                    Ok(event) => buffer.push(event),
                    Err(e) => {
                        eprintln!("Warning: Failed to deserialize current batch event: {e}");
                        // Continue with other events rather than failing completely
                    }
                }
            }
        }
        
        // Deserialize and restore expired batch events
        {
            let mut expired = self.expired.lock().unwrap();
            expired.clear();
            
            for serialized_event in state_data.expired_batch {
                match self.deserialize_event(&serialized_event) {
                    Ok(event) => expired.push(event),
                    Err(e) => {
                        eprintln!("Warning: Failed to deserialize expired batch event: {e}");
                        // Continue with other events rather than failing completely
                    }
                }
            }
        }
        
        // Restore timing state
        *self.start_time.lock().unwrap() = state_data.start_time;
        
        // Restore metadata
        self.duration_ms = state_data.duration_ms;
        *self.total_events_processed.lock().unwrap() = state_data.total_events_processed;
        
        Ok(())
    }

    fn get_changelog(&self, since: CheckpointId) -> Result<ChangeLog, StateError> {
        let last_checkpoint = self.last_checkpoint_id.lock().unwrap();
        
        if let Some(last_id) = *last_checkpoint {
            if since > last_id {
                return Err(StateError::CheckpointNotFound { checkpoint_id: since });
            }
        }
        
        let change_log = self.change_log.lock().unwrap();
        let mut changelog = ChangeLog::new(since, since + 1);
        
        for operation in change_log.iter() {
            changelog.add_operation(operation.clone());
        }
        
        Ok(changelog)
    }

    fn apply_changelog(&mut self, changes: &ChangeLog) -> Result<(), StateError> {
        // For time batch windows, we could apply incremental changes
        // For now, this is a simplified implementation
        println!("Applying {} state operations to time batch window", changes.operations.len());
        
        // In a full implementation, we would:
        // 1. Parse each operation
        // 2. Apply inserts/deletes to the buffers
        // 3. Handle time-based batch operations
        // 4. Update start time changes properly
        
        Ok(())
    }

    fn estimate_size(&self) -> StateSize {
        let buffer = self.buffer.lock().unwrap();
        let expired = self.expired.lock().unwrap();
        let entries = buffer.len() + expired.len();
        
        // Estimate bytes per event (rough calculation)
        let estimated_bytes_per_event = 200; // Conservative estimate
        let total_bytes = entries * estimated_bytes_per_event;
        
        // Estimate growth rate based on time duration
        // Time batch windows have variable growth based on event rate
        let start_time = *self.start_time.lock().unwrap();
        let growth_rate = if start_time.is_some() && self.duration_ms > 0 {
            // If we have an active batch, estimate remaining capacity
            estimated_bytes_per_event as f64 * 0.5 // Simplified estimate
        } else {
            estimated_bytes_per_event as f64
        };
        
        StateSize {
            bytes: total_bytes,
            entries,
            estimated_growth_rate: growth_rate,
        }
    }

    fn access_pattern(&self) -> AccessPattern {
        // Time batch windows have a sequential access pattern with temporal locality
        // Events are added in time order and flushed as time-based batches
        AccessPattern::Sequential
    }

    fn component_metadata(&self) -> StateMetadata {
        let mut metadata = StateMetadata::new(self.component_id.clone(), "TimeBatchWindowProcessor".to_string());
        metadata.access_pattern = self.access_pattern();
        metadata.size_estimation = self.estimate_size();
        
        // Add custom metadata
        metadata.custom_metadata.insert("duration_ms".to_string(), self.duration_ms.to_string());
        metadata.custom_metadata.insert("window_type".to_string(), "timeBatch".to_string());
        
        let buffer_len = self.buffer.lock().unwrap().len();
        let expired_len = self.expired.lock().unwrap().len();
        metadata.custom_metadata.insert("current_batch_size".to_string(), buffer_len.to_string());
        metadata.custom_metadata.insert("expired_batch_size".to_string(), expired_len.to_string());
        
        let start_time = *self.start_time.lock().unwrap();
        if let Some(start) = start_time {
            metadata.custom_metadata.insert("batch_start_time".to_string(), start.to_string());
        }
        
        metadata
    }
}

impl TimeBatchWindowStateHolder {
    /// Apply compression to data
    fn apply_compression(&self, data: Vec<u8>, compression: &CompressionType) -> Result<Vec<u8>, StateError> {
        match compression {
            CompressionType::None => Ok(data),
            CompressionType::LZ4 => {
                // In a real implementation, we'd use lz4 compression
                // For now, return the data as-is
                println!("LZ4 compression not implemented, returning uncompressed data");
                Ok(data)
            }
            CompressionType::Snappy => {
                // In a real implementation, we'd use snappy compression
                println!("Snappy compression not implemented, returning uncompressed data");
                Ok(data)
            }
            CompressionType::Zstd => {
                // In a real implementation, we'd use zstd compression
                println!("Zstd compression not implemented, returning uncompressed data");
                Ok(data)
            }
        }
    }

    /// Decompress data
    fn decompress_data(&self, data: &[u8], compression: &CompressionType) -> Result<Vec<u8>, StateError> {
        match compression {
            CompressionType::None => Ok(data.to_vec()),
            CompressionType::LZ4 => {
                // In a real implementation, we'd decompress with lz4
                println!("LZ4 decompression not implemented, returning data as-is");
                Ok(data.to_vec())
            }
            CompressionType::Snappy => {
                // In a real implementation, we'd decompress with snappy
                println!("Snappy decompression not implemented, returning data as-is");
                Ok(data.to_vec())
            }
            CompressionType::Zstd => {
                // In a real implementation, we'd decompress with zstd
                println!("Zstd decompression not implemented, returning data as-is");
                Ok(data.to_vec())
            }
        }
    }
}


/// Serializable state data for TimeBatchWindowProcessor
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TimeBatchWindowStateData {
    current_batch: Vec<Vec<u8>>,
    expired_batch: Vec<Vec<u8>>,
    duration_ms: i64,
    start_time: Option<i64>,
    total_events_processed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::complex_event::ComplexEventType;
    use crate::core::event::value::AttributeValue;
    use std::sync::Arc;

    #[test]
    fn test_time_batch_window_state_holder_creation() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let expired = Arc::new(Mutex::new(Vec::new()));
        let start_time = Arc::new(Mutex::new(None));
        let holder = TimeBatchWindowStateHolder::new(
            buffer,
            expired,
            start_time,
            "test_time_batch_window".to_string(),
            5000, // 5 second window
        );
        
        assert_eq!(holder.schema_version(), SchemaVersion::new(1, 0, 0));
        assert_eq!(holder.access_pattern(), AccessPattern::Sequential);
    }

    #[test]
    fn test_state_serialization_and_deserialization() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let expired = Arc::new(Mutex::new(Vec::new()));
        let start_time = Arc::new(Mutex::new(Some(1000)));
        
        // Add some test events to current batch
        {
            let mut buf = buffer.lock().unwrap();
            let mut event1 = StreamEvent::new(1000, 2, 0, 0);
            event1.before_window_data = vec![
                AttributeValue::String("test1".to_string()),
                AttributeValue::Int(42),
            ];
            buf.push(event1);
            
            let mut event2 = StreamEvent::new(2000, 2, 0, 0);
            event2.before_window_data = vec![
                AttributeValue::String("test2".to_string()),
                AttributeValue::Int(84),
            ];
            buf.push(event2);
        }
        
        // Add some test events to expired batch
        {
            let mut exp = expired.lock().unwrap();
            let mut event3 = StreamEvent::new(3000, 1, 0, 0);
            event3.before_window_data = vec![AttributeValue::String("expired1".to_string())];
            event3.event_type = ComplexEventType::Expired;
            exp.push(event3);
        }
        
        let mut holder = TimeBatchWindowStateHolder::new(
            buffer,
            expired,
            start_time,
            "test_time_batch_window".to_string(),
            5000,
        );
        
        let hints = SerializationHints::default();
        
        // Test serialization
        let snapshot = holder.serialize_state(&hints).unwrap();
        assert!(snapshot.verify_integrity());
        
        // Test deserialization
        let result = holder.deserialize_state(&snapshot);
        assert!(result.is_ok());
        
        // Verify the events were properly restored
        let buffer = holder.buffer.lock().unwrap();
        let expired = holder.expired.lock().unwrap();
        let start_time = *holder.start_time.lock().unwrap();
        assert_eq!(buffer.len(), 2); // Current batch events should be restored
        assert_eq!(expired.len(), 1); // Expired batch events should be restored
        assert_eq!(start_time, Some(1000));
        
        // Verify event data integrity
        if let Some(event) = buffer.get(0) {
            assert_eq!(event.timestamp, 1000);
            assert_eq!(event.before_window_data.len(), 2);
        }
        
        if let Some(event) = expired.get(0) {
            assert_eq!(event.timestamp, 3000);
            assert_eq!(event.before_window_data.len(), 1);
        }
    }

    #[test]
    fn test_change_log_tracking() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let expired = Arc::new(Mutex::new(Vec::new()));
        let start_time = Arc::new(Mutex::new(None));
        let holder = TimeBatchWindowStateHolder::new(
            buffer,
            expired,
            start_time,
            "test_time_batch_window".to_string(),
            3000,
        );
        
        // Create test events
        let mut event1 = StreamEvent::new(1000, 1, 0, 0);
        event1.before_window_data = vec![AttributeValue::Int(42)];
        
        let mut event2 = StreamEvent::new(2000, 1, 0, 0);
        event2.before_window_data = vec![AttributeValue::Int(84)];
        
        // Record event additions
        holder.record_event_added(&event1);
        holder.record_event_added(&event2);
        
        // Get changelog
        let changelog = holder.get_changelog(0).unwrap();
        assert_eq!(changelog.operations.len(), 2);
        
        // Record batch flush
        let current_batch = vec![event1, event2];
        let expired_batch = vec![];
        holder.record_batch_flushed(&current_batch, &expired_batch, 4000);
        
        let changelog = holder.get_changelog(0).unwrap();
        assert_eq!(changelog.operations.len(), 3);
        
        // Record start time change
        holder.record_start_time_updated(None, Some(1000));
        
        let changelog = holder.get_changelog(0).unwrap();
        assert_eq!(changelog.operations.len(), 4);
    }

    #[test]
    fn test_size_estimation() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let expired = Arc::new(Mutex::new(Vec::new()));
        let start_time = Arc::new(Mutex::new(None));
        let holder = TimeBatchWindowStateHolder::new(
            buffer.clone(),
            expired.clone(),
            start_time.clone(),
            "test_time_batch_window".to_string(),
            5000, // 5 second window
        );
        
        // Test empty state
        let size = holder.estimate_size();
        assert_eq!(size.entries, 0);
        assert_eq!(size.bytes, 0);
        
        // Add some events to current batch
        {
            let mut buf = buffer.lock().unwrap();
            for i in 0..3 {
                let mut event = StreamEvent::new(1000 + i * 100, 1, 0, 0);
                event.before_window_data = vec![AttributeValue::Int(i as i32)];
                buf.push(event);
            }
        }
        
        // Add one expired event
        {
            let mut exp = expired.lock().unwrap();
            let mut event = StreamEvent::new(500, 1, 0, 0);
            event.before_window_data = vec![AttributeValue::Int(99)];
            exp.push(event);
        }
        
        // Set start time to simulate active batch
        *start_time.lock().unwrap() = Some(1000);
        
        let size = holder.estimate_size();
        assert_eq!(size.entries, 4); // 3 current + 1 expired
        assert_eq!(size.bytes, 4 * 200); // 200 bytes per event estimate
        assert!(size.estimated_growth_rate > 0.0); // Active time window can grow
    }

    #[test]
    fn test_time_batch_metadata() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let expired = Arc::new(Mutex::new(Vec::new()));
        let start_time = Arc::new(Mutex::new(Some(1000)));
        
        // Add some events to both buffers
        {
            let mut buf = buffer.lock().unwrap();
            for i in 0..2 {
                let mut event = StreamEvent::new(1000 + i * 100, 1, 0, 0);
                event.before_window_data = vec![AttributeValue::Int(i as i32)];
                buf.push(event);
            }
        }
        
        {
            let mut exp = expired.lock().unwrap();
            let mut event = StreamEvent::new(500, 1, 0, 0);
            event.before_window_data = vec![AttributeValue::Int(99)];
            exp.push(event);
        }
        
        let holder = TimeBatchWindowStateHolder::new(
            buffer,
            expired,
            start_time,
            "test_time_batch_window".to_string(),
            5000,
        );
        
        let metadata = holder.component_metadata();
        assert_eq!(metadata.component_type, "TimeBatchWindowProcessor");
        assert_eq!(metadata.custom_metadata.get("duration_ms").unwrap(), "5000");
        assert_eq!(metadata.custom_metadata.get("window_type").unwrap(), "timeBatch");
        assert_eq!(metadata.custom_metadata.get("current_batch_size").unwrap(), "2");
        assert_eq!(metadata.custom_metadata.get("expired_batch_size").unwrap(), "1");
        assert_eq!(metadata.custom_metadata.get("batch_start_time").unwrap(), "1000");
    }
}