// Corresponds to package io.siddhi.query.api.execution.query.input

pub mod handler;
pub mod state;
pub mod store;
pub mod stream;

// Re-export key elements from the stream module as it defines the main InputStream enum/struct
pub use self::stream::{
    InputStream,
    SingleInputStream,
    JoinInputStream,
    StateInputStream,
    // BasicSingleInputStream, // Removed
    // AnonymousInputStream, // Removed
    JoinType, // Enum from JoinInputStream
    StateInputStreamType, // Enum from StateInputStream
    JoinEventTrigger, // Enum from JoinInputStream
};

// Re-export key elements from handler module
pub use self::handler::{
    StreamHandler, // Enum or Trait
    Filter,
    StreamFunction,
    WindowHandler, // Use the already aliased WindowHandler from handler/mod.rs
};

// Re-export key elements from state module
pub use self::state::{
    StateElement, // Enum or Trait
    StreamStateElement,
    LogicalStateElement,
    LogicalStateElementType, // Enum from LogicalStateElement
    NextStateElement,
    EveryStateElement,
    CountStateElement,
    AbsentStreamStateElement,
    State, // Utility struct with static methods
};

// Re-export key elements from store module
pub use self::store::{
    InputStore, // Trait
    Store,
    ConditionInputStore,
    AggregationInputStore,
};
