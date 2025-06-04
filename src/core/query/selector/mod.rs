// siddhi_rust/src/core/query/selector/mod.rs
pub mod attribute; // For OutputAttributeProcessor and future aggregators/processors
pub mod select_processor; // Corresponds to QuerySelector.java
// pub mod order_by_event_comparator; // For OrderByEventComparator.java
// pub mod group_by_key_generator; // For GroupByKeyGenerator.java

pub use self::select_processor::SelectProcessor;
pub use self::attribute::OutputAttributeProcessor; // Re-export for convenience

// Other components like OrderByEventComparator, GroupByKeyGenerator would be exported here.
