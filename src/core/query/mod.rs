// siddhi_rust/src/core/query/mod.rs

pub mod processor;
// Other query submodules will be added here: input, output, selector (core internal versions)
// pub mod input;
// pub mod output;
// pub mod selector;

// For top-level query runtime classes like QueryRuntime, OnDemandQueryRuntime
// pub mod query_runtime;
// pub mod on_demand_query_runtime;
// etc.

pub use self::processor::{Processor, ProcessingMode};
