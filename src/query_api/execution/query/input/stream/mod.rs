// Corresponds to package io.siddhi.query.api.execution.query.input.stream

// Individual stream types
// pub mod anonymous_input_stream; // Functionality merged into single_input_stream.rs
// pub mod basic_single_input_stream; // Functionality merged into single_input_stream.rs
pub mod input_stream; // Defines the main InputStream enum and its factory methods
pub mod join_input_stream;
pub mod single_input_stream;
pub mod state_input_stream;

// Re-export key types for easier access from parent modules (e.g., query_api::execution::query::input)
pub use self::input_stream::{InputStream, InputStreamTrait}; // This is the main enum & trait
pub use self::single_input_stream::{SingleInputStream, SingleInputStreamKind};
// BasicSingleInputStream and AnonymousInputStream are not top-level structs anymore.
// Their logic is within SingleInputStream or specific constructors.
// pub use self::basic_single_input_stream::BasicSingleInputStream; // Removed
// pub use self::anonymous_input_stream::AnonymousInputStream; // Removed
pub use self::join_input_stream::{JoinInputStream, Type as JoinType, EventTrigger as JoinEventTrigger}; // Removed Within from here
pub use crate::query_api::aggregation::Within as JoinWithin; // Import Within directly and alias
pub use self::state_input_stream::{StateInputStream, Type as StateInputStreamType};


// Comments updated:
// - `input_stream.rs` defines `pub enum InputStream { Single(SingleInputStream), Join(JoinInputStream), State(StateInputStream) }` and `InputStreamTrait`.
// - `single_input_stream.rs` defines `pub struct SingleInputStream { kind: SingleInputStreamKind }` where kind can be Basic or Anonymous.
// - `join_input_stream.rs` defines `pub struct JoinInputStream { ... }` and its enums `Type` and `EventTrigger`.
// - `state_input_stream.rs` defines `pub struct StateInputStream { ... }` and its enum `Type`.
// All structs that are variants or composed within these (e.g. SingleInputStream, JoinInputStream, StateInputStream)
// must compose `siddhi_element: SiddhiElement`.
// The `InputStream` enum implements `SiddhiElement` and `InputStreamTrait` by dispatching to its variants.
