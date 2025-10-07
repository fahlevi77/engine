// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod log_sink;
pub mod sink_factory;
pub mod sink_trait;

pub use log_sink::LogSink;
pub use sink_factory::{create_sink_from_stream_config, SinkFactoryRegistry};
pub use sink_trait::Sink;
