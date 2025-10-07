// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod output_stream; // Defines the main OutputStream enum (wrapping actions) and OutputEventType
pub mod ratelimit;
pub mod stream; // Added this module for specific stream actions like SetAttribute

pub use self::output_stream::{OutputEventType, OutputStream};
// SetAttributePlaceholder was removed from output_stream.rs. SetAttribute is now re-exported below.

pub use self::ratelimit::{
    EventsOutputRate, OutputRate, OutputRateBehavior, OutputRateVariant, SnapshotOutputRate,
    TimeOutputRate,
};
pub use self::stream::{SetAttribute, UpdateSet}; // Re-exporting SetAttribute and UpdateSet
