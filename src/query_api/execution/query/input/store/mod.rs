// Corresponds to package io.siddhi.query.api.execution.query.input.store

pub mod aggregation_input_store;
pub mod condition_input_store;
pub mod input_store; // Interface, will be a trait or enum
pub mod store;       // This is a BasicSingleInputStream that also implements InputStore

// Re-export key types
pub use self::input_store::InputStore; // This will be the main enum for different store input types
pub use self::aggregation_input_store::AggregationInputStore;
pub use self::condition_input_store::ConditionInputStore;
pub use self::store::Store; // The Store struct that acts as a specific InputStore type.

// InputStore will likely be an enum:
// pub enum InputStoreType { // Renaming from InputStore to avoid conflict if InputStore is a trait
//     Condition(ConditionInputStore),
//     Aggregation(AggregationInputStore),
//     // Store itself is a kind of BasicSingleInputStream, so it might not be a variant here
//     // if ConditionInputStore and AggregationInputStore directly embed Store.
// }
// The InputStore.java interface has static factory methods for `Store`.
// It also has getStoreReferenceId() and getStoreId().
// Store.java extends BasicSingleInputStream and implements InputStore.
// It has methods `on(Expression)` returning ConditionInputStore, and `on(Within, Per)` returning AggregationInputStore.
// This suggests Store is the primary entry point.

// Let's define InputStore as a trait for now.
// ConditionInputStore and AggregationInputStore will implement it.
// Store struct will also implement it.
// The methods on(Expression) etc. on Store struct will then return these concrete types.
