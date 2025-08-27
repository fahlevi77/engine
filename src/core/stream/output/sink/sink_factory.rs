//! Sink Factory Implementation
//! 
//! Provides automatic sink creation from configuration

use std::sync::Arc;
use std::collections::HashMap;
use crate::core::stream::output::sink::{Sink, LogSink};
use crate::core::stream::output::stream_callback::StreamCallback;
use crate::core::config::{ProcessorConfigReader, types::application_config::SinkConfig};

/// Registry for sink factories
pub struct SinkFactoryRegistry {
    factories: HashMap<String, Box<dyn SinkFactoryTrait>>,
}

impl SinkFactoryRegistry {
    /// Create a new sink factory registry with default factories
    pub fn new() -> Self {
        let mut registry = Self {
            factories: HashMap::new(),
        };
        
        // Register default sink factories
        registry.register("log", Box::new(LogSinkFactory));
        // Future: registry.register("kafka", Box::new(KafkaSinkFactory));
        // Future: registry.register("http", Box::new(HttpSinkFactory));
        
        registry
    }
    
    /// Register a new sink factory
    pub fn register(&mut self, sink_type: &str, factory: Box<dyn SinkFactoryTrait>) {
        self.factories.insert(sink_type.to_string(), factory);
    }
    
    /// Create a sink from configuration
    pub fn create_sink(
        &self,
        sink_config: &SinkConfig,
        config_reader: Option<Arc<ProcessorConfigReader>>,
        sink_name: &str,
    ) -> Result<Box<dyn StreamCallback>, String> {
        let factory = self.factories
            .get(&sink_config.sink_type)
            .ok_or_else(|| format!("Unknown sink type: {}", sink_config.sink_type))?;
        
        factory.create_from_config(sink_config, config_reader, sink_name)
    }
}

/// Trait for sink factories
pub trait SinkFactoryTrait: Send + Sync {
    /// Create a sink from configuration
    fn create_from_config(
        &self,
        sink_config: &SinkConfig,
        config_reader: Option<Arc<ProcessorConfigReader>>,
        sink_name: &str,
    ) -> Result<Box<dyn StreamCallback>, String>;
}

/// Factory for LogSink
struct LogSinkFactory;

impl SinkFactoryTrait for LogSinkFactory {
    fn create_from_config(
        &self,
        _sink_config: &SinkConfig,
        config_reader: Option<Arc<ProcessorConfigReader>>,
        sink_name: &str,
    ) -> Result<Box<dyn StreamCallback>, String> {
        // Create LogSink with configuration support
        let log_sink = LogSink::new_with_config(
            config_reader,
            sink_name.to_string(),
        );
        
        Ok(Box::new(log_sink))
    }
}

/// Helper function to create a sink from a stream configuration
pub fn create_sink_from_stream_config(
    stream_name: &str,
    sink_config: &SinkConfig,
    config_reader: Option<Arc<ProcessorConfigReader>>,
) -> Result<Box<dyn StreamCallback>, String> {
    // Use the global registry (in a real implementation, this would be passed in)
    let registry = SinkFactoryRegistry::new();
    registry.create_sink(sink_config, config_reader, stream_name)
}