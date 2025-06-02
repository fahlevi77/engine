pub mod ratelimit;
pub mod output_stream; // Defines the main OutputStream enum (wrapping actions) and OutputEventType
pub mod stream; // Added this module for specific stream actions like SetAttribute

pub use self::output_stream::{OutputStream, OutputEventType};
// SetAttributePlaceholder was removed from output_stream.rs. SetAttribute is now re-exported below.

pub use self::ratelimit::{OutputRate, OutputRateVariant, OutputRateBehavior, EventsOutputRate, TimeOutputRate, SnapshotOutputRate};
pub use self::stream::{SetAttribute, UpdateSet}; // Re-exporting SetAttribute and UpdateSet
