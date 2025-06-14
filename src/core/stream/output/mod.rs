// siddhi_rust/src/core/stream/output/mod.rs

pub mod sink;
pub mod stream_callback; // For sink implementations

pub use self::sink::{LogSink, Sink};
pub use self::stream_callback::{LogStreamCallback, StreamCallback};
