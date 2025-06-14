// Corresponds to package io.siddhi.query.api.execution.query.input

pub mod handler;
pub mod state;
pub mod store;
pub mod stream;

// Re-export key elements from the stream module as it defines the main InputStream enum/struct
pub use self::stream::{
    InputStream,
    JoinEventTrigger, // Enum from JoinInputStream
    JoinInputStream,
    // BasicSingleInputStream, // Removed
    // AnonymousInputStream, // Removed
    JoinType, // Enum from JoinInputStream
    SingleInputStream,
    StateInputStream,
    StateInputStreamType, // Enum from StateInputStream
};

// Re-export key elements from handler module
pub use self::handler::{
    Filter,
    StreamFunction,
    StreamHandler, // Enum or Trait
    WindowHandler, // Use the already aliased WindowHandler from handler/mod.rs
};

// Re-export key elements from state module
pub use self::state::{
    AbsentStreamStateElement,
    CountStateElement,
    EveryStateElement,
    LogicalStateElement,
    LogicalStateElementType, // Enum from LogicalStateElement
    NextStateElement,
    State,        // Utility struct with static methods
    StateElement, // Enum or Trait
    StreamStateElement,
};

// Re-export key elements from store module
pub use self::store::{
    AggregationInputStore,
    ConditionInputStore,
    InputStore, // Trait
    Store,
};
