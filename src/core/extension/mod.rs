use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::core::stream::output::stream_callback::StreamCallback;

use crate::core::config::{siddhi_app_context::SiddhiAppContext, siddhi_query_context::SiddhiQueryContext};
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::query::processor::Processor;
use crate::core::query::selector::attribute::aggregator::AttributeAggregatorExecutor;
use crate::query_api::execution::query::input::handler::WindowHandler;

pub trait WindowProcessorFactory: Debug + Send + Sync {
    fn create(
        &self,
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Arc<Mutex<dyn Processor>>, String>;
    fn clone_box(&self) -> Box<dyn WindowProcessorFactory>;
}
impl Clone for Box<dyn WindowProcessorFactory> {
    fn clone(&self) -> Self { self.clone_box() }
}

pub trait AttributeAggregatorFactory: Debug + Send + Sync {
    fn create(&self) -> Box<dyn AttributeAggregatorExecutor>;
    fn clone_box(&self) -> Box<dyn AttributeAggregatorFactory>;
}
impl Clone for Box<dyn AttributeAggregatorFactory> {
    fn clone(&self) -> Self { self.clone_box() }
}

pub trait SourceFactory: Debug + Send + Sync {
    fn create(&self) -> Box<dyn crate::core::stream::input::source::Source>;
    fn clone_box(&self) -> Box<dyn SourceFactory>;
}
impl Clone for Box<dyn SourceFactory> { fn clone(&self) -> Self { self.clone_box() } }

pub trait SinkFactory: Debug + Send + Sync {
    fn create(&self) -> Box<dyn crate::core::stream::output::sink::Sink>;
    fn clone_box(&self) -> Box<dyn SinkFactory>;
}
impl Clone for Box<dyn SinkFactory> { fn clone(&self) -> Self { self.clone_box() } }

pub trait StoreFactory: Debug + Send + Sync {
    fn clone_box(&self) -> Box<dyn StoreFactory>;
}
impl Clone for Box<dyn StoreFactory> { fn clone(&self) -> Self { self.clone_box() } }

pub trait SourceMapperFactory: Debug + Send + Sync {
    fn clone_box(&self) -> Box<dyn SourceMapperFactory>;
}
impl Clone for Box<dyn SourceMapperFactory> { fn clone(&self) -> Self { self.clone_box() } }

pub trait SinkMapperFactory: Debug + Send + Sync {
    fn clone_box(&self) -> Box<dyn SinkMapperFactory>;
}
impl Clone for Box<dyn SinkMapperFactory> { fn clone(&self) -> Self { self.clone_box() } }

pub trait TableFactory: Debug + Send + Sync {
    fn create(
        &self,
        table_name: String,
        properties: std::collections::HashMap<String, String>,
        ctx: Arc<crate::core::config::siddhi_context::SiddhiContext>,
    ) -> Result<Arc<dyn crate::core::table::Table>, String>;
    fn clone_box(&self) -> Box<dyn TableFactory>;
}
impl Clone for Box<dyn TableFactory> { fn clone(&self) -> Self { self.clone_box() } }

#[derive(Debug, Clone)]
pub struct TimerSourceFactory;

impl SourceFactory for TimerSourceFactory {
    fn create(&self) -> Box<dyn crate::core::stream::input::source::Source> {
        Box::new(crate::core::stream::input::source::timer_source::TimerSource::new(1000))
    }
    fn clone_box(&self) -> Box<dyn SourceFactory> { Box::new(self.clone()) }
}

#[derive(Debug, Clone)]
pub struct LogSinkFactory;

impl SinkFactory for LogSinkFactory {
    fn create(&self) -> Box<dyn crate::core::stream::output::sink::Sink> {
        Box::new(crate::core::stream::output::sink::LogSink::new())
    }
    fn clone_box(&self) -> Box<dyn SinkFactory> { Box::new(self.clone()) }
}
