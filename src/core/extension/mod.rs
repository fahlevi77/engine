use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::core::function::script::Script;
use crate::core::store::Store;
use crate::core::stream::input::mapper::SourceMapper;
use crate::core::stream::output::mapper::SinkMapper;

use crate::core::config::{
    eventflux_app_context::EventFluxAppContext, eventflux_query_context::EventFluxQueryContext,
};
use crate::core::query::processor::Processor;
use crate::core::query::selector::attribute::aggregator::AttributeAggregatorExecutor;
use crate::query_api::execution::query::input::handler::WindowHandler;

pub trait WindowProcessorFactory: Debug + Send + Sync {
    fn name(&self) -> &'static str;
    fn create(
        &self,
        handler: &WindowHandler,
        app_ctx: Arc<EventFluxAppContext>,
        query_ctx: Arc<EventFluxQueryContext>,
    ) -> Result<Arc<Mutex<dyn Processor>>, String>;
    fn clone_box(&self) -> Box<dyn WindowProcessorFactory>;
}
impl Clone for Box<dyn WindowProcessorFactory> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait AttributeAggregatorFactory: Debug + Send + Sync {
    fn name(&self) -> &'static str;
    fn create(&self) -> Box<dyn AttributeAggregatorExecutor>;
    fn clone_box(&self) -> Box<dyn AttributeAggregatorFactory>;
}
impl Clone for Box<dyn AttributeAggregatorFactory> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait SourceFactory: Debug + Send + Sync {
    fn name(&self) -> &'static str;
    fn create(&self) -> Box<dyn crate::core::stream::input::source::Source>;
    fn clone_box(&self) -> Box<dyn SourceFactory>;
}
impl Clone for Box<dyn SourceFactory> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait SinkFactory: Debug + Send + Sync {
    fn name(&self) -> &'static str;
    fn create(&self) -> Box<dyn crate::core::stream::output::sink::Sink>;
    fn clone_box(&self) -> Box<dyn SinkFactory>;
}
impl Clone for Box<dyn SinkFactory> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait StoreFactory: Debug + Send + Sync {
    fn name(&self) -> &'static str;
    fn create(&self) -> Box<dyn Store>;
    fn clone_box(&self) -> Box<dyn StoreFactory>;
}
impl Clone for Box<dyn StoreFactory> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait SourceMapperFactory: Debug + Send + Sync {
    fn name(&self) -> &'static str;
    fn create(&self) -> Box<dyn SourceMapper>;
    fn clone_box(&self) -> Box<dyn SourceMapperFactory>;
}
impl Clone for Box<dyn SourceMapperFactory> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait SinkMapperFactory: Debug + Send + Sync {
    fn name(&self) -> &'static str;
    fn create(&self) -> Box<dyn SinkMapper>;
    fn clone_box(&self) -> Box<dyn SinkMapperFactory>;
}
impl Clone for Box<dyn SinkMapperFactory> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait TableFactory: Debug + Send + Sync {
    fn name(&self) -> &'static str;
    fn create(
        &self,
        table_name: String,
        properties: std::collections::HashMap<String, String>,
        ctx: Arc<crate::core::config::eventflux_context::EventFluxContext>,
    ) -> Result<Arc<dyn crate::core::table::Table>, String>;
    fn clone_box(&self) -> Box<dyn TableFactory>;
}
impl Clone for Box<dyn TableFactory> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait ScriptFactory: Debug + Send + Sync {
    fn name(&self) -> &'static str;
    fn create(&self) -> Box<dyn Script>;
    fn clone_box(&self) -> Box<dyn ScriptFactory>;
}
impl Clone for Box<dyn ScriptFactory> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct TimerSourceFactory;

impl SourceFactory for TimerSourceFactory {
    fn name(&self) -> &'static str {
        "timer"
    }
    fn create(&self) -> Box<dyn crate::core::stream::input::source::Source> {
        Box::new(crate::core::stream::input::source::timer_source::TimerSource::new(1000))
    }
    fn clone_box(&self) -> Box<dyn SourceFactory> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct LogSinkFactory;

impl SinkFactory for LogSinkFactory {
    fn name(&self) -> &'static str {
        "log"
    }
    fn create(&self) -> Box<dyn crate::core::stream::output::sink::Sink> {
        Box::new(crate::core::stream::output::sink::LogSink::new())
    }
    fn clone_box(&self) -> Box<dyn SinkFactory> {
        Box::new(self.clone())
    }
}

/// FFI callback type used when dynamically loading extensions.
pub type RegisterFn = unsafe extern "C" fn(&crate::core::eventflux_manager::EventFluxManager);

/// Symbol names looked up by [`EventFluxManager::set_extension`].
pub const REGISTER_EXTENSION_FN: &[u8] = b"register_extension";
pub const REGISTER_WINDOWS_FN: &[u8] = b"register_windows";
pub const REGISTER_FUNCTIONS_FN: &[u8] = b"register_functions";
pub const REGISTER_SOURCES_FN: &[u8] = b"register_sources";
pub const REGISTER_SINKS_FN: &[u8] = b"register_sinks";
pub const REGISTER_STORES_FN: &[u8] = b"register_stores";
pub const REGISTER_SOURCE_MAPPERS_FN: &[u8] = b"register_source_mappers";
pub const REGISTER_SINK_MAPPERS_FN: &[u8] = b"register_sink_mappers";
