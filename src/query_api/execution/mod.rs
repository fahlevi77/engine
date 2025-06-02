pub mod query;
pub mod execution_element;
pub mod partition;

// Re-export top-level executable units
pub use self::execution_element::{ExecutionElement, ExecutionElementTrait}; // Assuming ExecutionElementTrait exists
pub use self::query::Query;
// If OnDemandQuery can be a top-level execution element (it's usually part of a StoreQuery or used directly by API)
// pub use self::query::OnDemandQuery;
pub use self::partition::Partition;

// Re-export common partition types for convenience
pub use self::partition::{PartitionType, PartitionTypeVariant, RangePartitionType, ValuePartitionType, RangePartitionProperty};

// Re-export key sub-modules or types from query if they are frequently accessed via `execution::`
pub use self::query::input;
pub use self::query::output;
pub use self::query::selection;
// For example, to allow `execution::input::InputStream`
// or `execution::output::OutputStream`
// or `execution::selection::Selector`
```
