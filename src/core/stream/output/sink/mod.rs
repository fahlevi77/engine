pub mod log_sink;
pub mod sink_trait;
pub mod sink_factory;

pub use log_sink::LogSink;
pub use sink_trait::Sink;
pub use sink_factory::{SinkFactoryRegistry, create_sink_from_stream_config};
