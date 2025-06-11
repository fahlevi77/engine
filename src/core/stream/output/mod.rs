// siddhi_rust/src/core/stream/output/mod.rs

pub mod stream_callback;
pub mod sink; // For sink implementations

pub use self::stream_callback::{StreamCallback, LogStreamCallback};
pub use self::sink::LogSink;
