// Corresponds to io.siddhi.core.config.SiddhiContext
use std::collections::HashMap;
use std::sync::{Arc, RwLockReadGuard}; // For shared, thread-safe components, Added RwLockReadGuard

use super::statistics_configuration::StatisticsConfiguration;

// --- Placeholders for complex types/trait objects ---
// These would be defined in their respective modules (e.g., util::persistence, extension, etc.)
// For now, using simple struct placeholders or type aliases.

// pub trait Extension {} // Example
// pub type ExtensionType = Box<dyn Extension + Send + Sync>; // Example for dynamic dispatch
pub type ExtensionClassPlaceholder = String; // Simplified: Class name as String

// pub trait DataSource {} // Example
// pub type DataSourceType = Arc<dyn DataSource + Send + Sync>; // Example
pub type DataSourcePlaceholder = String; // Simplified

// pub trait PersistenceStore {} // Example
// pub type PersistenceStoreType = Arc<dyn PersistenceStore + Send + Sync>;
pub type PersistenceStorePlaceholder = String; // Simplified

// pub trait IncrementalPersistenceStore {} // Example
// pub type IncrementalPersistenceStoreType = Arc<dyn IncrementalPersistenceStore + Send + Sync>;
pub type IncrementalPersistenceStorePlaceholder = String; // Simplified

use crate::core::exception::error_store::ErrorStore;

use crate::core::persistence::data_source::DataSource; // Using actual DataSource trait
// pub trait ConfigManager {} // Example
// pub type ConfigManagerType = Arc<dyn ConfigManager + Send + Sync>;
pub type ConfigManagerPlaceholder = String; // Simplified

// pub trait SinkHandlerManager {} // Example
// pub type SinkHandlerManagerType = Arc<dyn SinkHandlerManager + Send + Sync>;
pub type SinkHandlerManagerPlaceholder = String; // Simplified

// pub trait SourceHandlerManager {} // Example
// pub type SourceHandlerManagerType = Arc<dyn SourceHandlerManager + Send + Sync>;
pub type SourceHandlerManagerPlaceholder = String; // Simplified

// pub trait RecordTableHandlerManager {} // Example
// pub type RecordTableHandlerManagerType = Arc<dyn RecordTableHandlerManager + Send + Sync>;
pub type RecordTableHandlerManagerPlaceholder = String; // Simplified

// pub trait AbstractExtensionHolder {} // Example
// pub type AbstractExtensionHolderType = Arc<dyn AbstractExtensionHolder + Send + Sync>;
pub type AbstractExtensionHolderPlaceholder = String; // Simplified

// Using String for attributes for now, could be an enum or serde_json::Value
pub type AttributeValuePlaceholder = String;

// Placeholder for Disruptor's ExceptionHandler<Object>
pub type DisruptorExceptionHandlerPlaceholder = String;


use crate::core::executor::function::ScalarFunctionExecutor; // Added
use std::sync::RwLock; // Added for scalar_function_factories and attributes, data_sources
use crate::core::table::Table;

/// Shared context for all Siddhi Apps in a SiddhiManager instance.
#[derive(Debug)] // Default will be custom, Clone removed due to Arc<RwLock<...>> direct Clone
pub struct SiddhiContext {
    pub statistics_configuration: StatisticsConfiguration,
    attributes: Arc<RwLock<HashMap<String, AttributeValuePlaceholder>>>,

    // --- Placeholders mirroring Java fields ---
    siddhi_extensions: HashMap<String, ExtensionClassPlaceholder>,
    deprecated_siddhi_extensions: HashMap<String, ExtensionClassPlaceholder>,
    persistence_store: Option<PersistenceStorePlaceholder>,
    incremental_persistence_store: Option<IncrementalPersistenceStorePlaceholder>,
    error_store: Option<Arc<dyn ErrorStore>>, 
    extension_holder_map: HashMap<String, AbstractExtensionHolderPlaceholder>,
    config_manager: ConfigManagerPlaceholder,
    sink_handler_manager: Option<SinkHandlerManagerPlaceholder>,
    source_handler_manager: Option<SourceHandlerManagerPlaceholder>,
    record_table_handler_manager: Option<RecordTableHandlerManagerPlaceholder>,
    default_disrupter_exception_handler: DisruptorExceptionHandlerPlaceholder,

    // Actual fields for extensions and data sources
    /// Stores factories for User-Defined Scalar Functions. Key: "namespace:name" or "name", Value: clonable factory instance.
    scalar_function_factories: Arc<RwLock<HashMap<String, Box<dyn ScalarFunctionExecutor>>>>,
    /// Window processor factories
    window_factories: Arc<RwLock<HashMap<String, Box<dyn crate::core::extension::WindowProcessorFactory>>>>,
    /// Attribute aggregator factories
    attribute_aggregator_factories: Arc<RwLock<HashMap<String, Box<dyn crate::core::extension::AttributeAggregatorFactory>>>>,
    /// Source factories
    source_factories: Arc<RwLock<HashMap<String, Box<dyn crate::core::extension::SourceFactory>>>>,
    /// Sink factories
    sink_factories: Arc<RwLock<HashMap<String, Box<dyn crate::core::extension::SinkFactory>>>>,
    /// Store factories
    store_factories: Arc<RwLock<HashMap<String, Box<dyn crate::core::extension::StoreFactory>>>>,
    /// Source mapper factories
    source_mapper_factories: Arc<RwLock<HashMap<String, Box<dyn crate::core::extension::SourceMapperFactory>>>>,
    /// Sink mapper factories
    sink_mapper_factories: Arc<RwLock<HashMap<String, Box<dyn crate::core::extension::SinkMapperFactory>>>>,
    /// Stores registered data sources. Key: data source name.
    data_sources: Arc<RwLock<HashMap<String, Arc<dyn DataSource>>>>,
    /// Registered tables available for queries.
    tables: Arc<RwLock<HashMap<String, Arc<dyn Table>>>>,

    pub dummy_field_siddhi_context_extensions: String,
}

impl SiddhiContext {
    pub fn new() -> Self {
        let mut ctx = Self {
            statistics_configuration: StatisticsConfiguration::default(),
            attributes: Arc::new(RwLock::new(HashMap::new())),
            scalar_function_factories: Arc::new(RwLock::new(HashMap::new())),
            window_factories: Arc::new(RwLock::new(HashMap::new())),
            attribute_aggregator_factories: Arc::new(RwLock::new(HashMap::new())),
            source_factories: Arc::new(RwLock::new(HashMap::new())),
            sink_factories: Arc::new(RwLock::new(HashMap::new())),
            store_factories: Arc::new(RwLock::new(HashMap::new())),
            source_mapper_factories: Arc::new(RwLock::new(HashMap::new())),
            sink_mapper_factories: Arc::new(RwLock::new(HashMap::new())),
            data_sources: Arc::new(RwLock::new(HashMap::new())),
            tables: Arc::new(RwLock::new(HashMap::new())),
            siddhi_extensions: HashMap::new(),
            deprecated_siddhi_extensions: HashMap::new(),
            persistence_store: None,
            incremental_persistence_store: None,
            error_store: None,
            extension_holder_map: HashMap::new(),
            config_manager: "InMemoryConfigManager_Placeholder".to_string(),
            sink_handler_manager: None,
            source_handler_manager: None,
            record_table_handler_manager: None,
            default_disrupter_exception_handler: "DefaultDisruptorExceptionHandler".to_string(),
            dummy_field_siddhi_context_extensions: String::new(),
        };
        ctx.register_default_extensions();
        ctx
    }

    // Example methods translated (simplified)
    pub fn get_siddhi_extensions(&self) -> &HashMap<String, String> {
        &self.siddhi_extensions
    }

    pub fn get_persistence_store(&self) -> Option<&String> {
        self.persistence_store.as_ref()
        // self.persistence_store.as_ref()
    }

    pub fn set_persistence_store(&mut self, persistence_store: String) {
        // Mirroring Java logic: only one persistence store allowed
        if self.incremental_persistence_store.is_some() {
            // In a full implementation this would return an error type
            panic!("Only one type of persistence store can exist. Incremental persistence store already registered!");
        }
        self.persistence_store = Some(persistence_store);
        // self.persistence_store = Some(persistence_store);
    }

    pub fn get_incremental_persistence_store(&self) -> Option<&String> {
        self.incremental_persistence_store.as_ref()
    }

    pub fn set_incremental_persistence_store(&mut self, store: String) {
        if self.persistence_store.is_some() {
            panic!("Only one type of persistence store can exist. Persistence store already registered!");
        }
        self.incremental_persistence_store = Some(store);
    }

    pub fn get_error_store(&self) -> Option<Arc<dyn ErrorStore>> {
        self.error_store.as_ref().cloned()
    }

    pub fn set_error_store(&mut self, store: Arc<dyn ErrorStore>) {
        self.error_store = Some(store);
    }

    // ... other getters and setters for various managers and stores ...

    pub fn get_data_source(&self, name: &str) -> Option<Arc<dyn DataSource>> {
        self.data_sources.read().unwrap().get(name).cloned()
    }

    pub fn add_data_source(&self, name: String, data_source: Arc<dyn DataSource>) {
        self.data_sources.write().unwrap().insert(name, data_source);
    }

    pub fn add_table(&self, name: String, table: Arc<dyn Table>) {
        self.tables.write().unwrap().insert(name, table);
    }

    pub fn get_table(&self, name: &str) -> Option<Arc<dyn Table>> {
        self.tables.read().unwrap().get(name).cloned()
    }

    pub fn get_statistics_configuration(&self) -> &StatisticsConfiguration {
        &self.statistics_configuration
    }

    pub fn set_statistics_configuration(&mut self, stat_config: StatisticsConfiguration) {
        self.statistics_configuration = stat_config;
    }

    pub fn get_config_manager(&self) -> &String {
        &self.config_manager
        // &self.config_manager
    }

    pub fn set_config_manager(&mut self, config_manager: String) {
        self.config_manager = config_manager;
        // self.config_manager = config_manager;
    }

    pub fn get_sink_handler_manager(&self) -> Option<&String> {
        self.sink_handler_manager.as_ref()
    }

    pub fn set_sink_handler_manager(&mut self, manager: String) {
        self.sink_handler_manager = Some(manager);
    }

    pub fn get_source_handler_manager(&self) -> Option<&String> {
        self.source_handler_manager.as_ref()
    }

    pub fn set_source_handler_manager(&mut self, manager: String) {
        self.source_handler_manager = Some(manager);
    }

    pub fn get_record_table_handler_manager(&self) -> Option<&String> {
        self.record_table_handler_manager.as_ref()
    }

    pub fn set_record_table_handler_manager(&mut self, manager: String) {
        self.record_table_handler_manager = Some(manager);
    }

    pub fn get_default_disruptor_exception_handler(&self) -> &String {
        &self.default_disrupter_exception_handler
    }

    pub fn get_attributes(&self) -> RwLockReadGuard<'_, HashMap<String, AttributeValuePlaceholder>> { // Return guard
        self.attributes.read().unwrap()
    }

    pub fn set_attribute(&self, key: String, value: AttributeValuePlaceholder) { // Takes &self due to RwLock
        self.attributes.write().unwrap().insert(key, value);
    }

    // --- Scalar Function Methods ---
    pub fn add_scalar_function_factory(&self, name: String, function_factory: Box<dyn ScalarFunctionExecutor>) {
        self.scalar_function_factories.write().unwrap().insert(name, function_factory);
    }

    pub fn get_scalar_function_factory(&self, name: &str) -> Option<Box<dyn ScalarFunctionExecutor>> {
        // .clone() on Box<dyn ScalarFunctionExecutor> calls the trait's clone_scalar_function()
        self.scalar_function_factories.read().unwrap().get(name).cloned()
    }

    fn register_default_extensions(&mut self) {
        use crate::core::query::processor::stream::window::{LengthWindowFactory, TimeWindowFactory};
        use crate::core::query::selector::attribute::aggregator::{
            SumAttributeAggregatorFactory, AvgAttributeAggregatorFactory, CountAttributeAggregatorFactory,
            DistinctCountAttributeAggregatorFactory, MinAttributeAggregatorFactory, MaxAttributeAggregatorFactory,
            MinForeverAttributeAggregatorFactory, MaxForeverAttributeAggregatorFactory,
        };

        self.add_window_factory("length".to_string(), Box::new(LengthWindowFactory));
        self.add_window_factory("time".to_string(), Box::new(TimeWindowFactory));

        self.add_attribute_aggregator_factory("sum".to_string(), Box::new(SumAttributeAggregatorFactory));
        self.add_attribute_aggregator_factory("avg".to_string(), Box::new(AvgAttributeAggregatorFactory));
        self.add_attribute_aggregator_factory("count".to_string(), Box::new(CountAttributeAggregatorFactory));
        self.add_attribute_aggregator_factory("distinctCount".to_string(), Box::new(DistinctCountAttributeAggregatorFactory));
        self.add_attribute_aggregator_factory("min".to_string(), Box::new(MinAttributeAggregatorFactory));
        self.add_attribute_aggregator_factory("max".to_string(), Box::new(MaxAttributeAggregatorFactory));
        self.add_attribute_aggregator_factory("minForever".to_string(), Box::new(MinForeverAttributeAggregatorFactory));
        self.add_attribute_aggregator_factory("maxForever".to_string(), Box::new(MaxForeverAttributeAggregatorFactory));
    }

    // --- Extension Factory Methods ---
    pub fn add_window_factory(&self, name: String, factory: Box<dyn crate::core::extension::WindowProcessorFactory>) {
        self.window_factories.write().unwrap().insert(name, factory);
    }

    pub fn get_window_factory(&self, name: &str) -> Option<Box<dyn crate::core::extension::WindowProcessorFactory>> {
        self.window_factories.read().unwrap().get(name).cloned()
    }

    pub fn add_attribute_aggregator_factory(&self, name: String, factory: Box<dyn crate::core::extension::AttributeAggregatorFactory>) {
        self.attribute_aggregator_factories.write().unwrap().insert(name, factory);
    }

    pub fn get_attribute_aggregator_factory(&self, name: &str) -> Option<Box<dyn crate::core::extension::AttributeAggregatorFactory>> {
        self.attribute_aggregator_factories.read().unwrap().get(name).cloned()
    }

    pub fn add_source_factory(&self, name: String, factory: Box<dyn crate::core::extension::SourceFactory>) {
        self.source_factories.write().unwrap().insert(name, factory);
    }
    pub fn add_sink_factory(&self, name: String, factory: Box<dyn crate::core::extension::SinkFactory>) {
        self.sink_factories.write().unwrap().insert(name, factory);
    }
    pub fn add_store_factory(&self, name: String, factory: Box<dyn crate::core::extension::StoreFactory>) {
        self.store_factories.write().unwrap().insert(name, factory);
    }
    pub fn add_source_mapper_factory(&self, name: String, factory: Box<dyn crate::core::extension::SourceMapperFactory>) {
        self.source_mapper_factories.write().unwrap().insert(name, factory);
    }
    pub fn add_sink_mapper_factory(&self, name: String, factory: Box<dyn crate::core::extension::SinkMapperFactory>) {
        self.sink_mapper_factories.write().unwrap().insert(name, factory);
    }
}

impl Default for SiddhiContext {
    fn default() -> Self {
        Self::new()
    }
}

// Clone implementation for SiddhiContext needs to handle Arc fields by cloning the Arc, not the underlying data.
impl Clone for SiddhiContext {
    fn clone(&self) -> Self {
        Self {
            statistics_configuration: self.statistics_configuration.clone(),
            attributes: Arc::clone(&self.attributes),
            scalar_function_factories: Arc::clone(&self.scalar_function_factories),
            window_factories: Arc::clone(&self.window_factories),
            attribute_aggregator_factories: Arc::clone(&self.attribute_aggregator_factories),
            source_factories: Arc::clone(&self.source_factories),
            sink_factories: Arc::clone(&self.sink_factories),
            store_factories: Arc::clone(&self.store_factories),
            source_mapper_factories: Arc::clone(&self.source_mapper_factories),
            sink_mapper_factories: Arc::clone(&self.sink_mapper_factories),
            data_sources: Arc::clone(&self.data_sources),
            tables: Arc::clone(&self.tables),
            siddhi_extensions: self.siddhi_extensions.clone(),
            deprecated_siddhi_extensions: self.deprecated_siddhi_extensions.clone(),
            persistence_store: self.persistence_store.clone(),
            incremental_persistence_store: self.incremental_persistence_store.clone(),
            error_store: self.error_store.clone(),
            extension_holder_map: self.extension_holder_map.clone(),
            config_manager: self.config_manager.clone(),
            sink_handler_manager: self.sink_handler_manager.clone(),
            source_handler_manager: self.source_handler_manager.clone(),
            record_table_handler_manager: self.record_table_handler_manager.clone(),
            default_disrupter_exception_handler: self.default_disrupter_exception_handler.clone(),
            dummy_field_siddhi_context_extensions: self.dummy_field_siddhi_context_extensions.clone(),
        }
    }
}
