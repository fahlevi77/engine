// siddhi_rust/src/core/stream/output/mod.rs

pub mod stream_callback;
// pub mod sink; // For sub-package sink/

pub use self::stream_callback::{StreamCallback, LogStreamCallback}; // Added LogStreamCallback
