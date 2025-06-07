// siddhi_rust/src/core/siddhi_app_runtime_builder.rs
use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::stream::stream_junction::StreamJunction;
use crate::core::query::query_runtime::QueryRuntime;
use crate::core::siddhi_app_runtime::SiddhiAppRuntime; // Actual SiddhiAppRuntime
use crate::core::window::WindowRuntime;
use crate::query_api::siddhi_app::SiddhiApp as ApiSiddhiApp; // For build() method arg
use crate::query_api::definition::StreamDefinition as ApiStreamDefinition; // Added this import

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Placeholders for runtime components until they are defined
#[derive(Debug, Clone, Default)] pub struct TableRuntimePlaceholder {}
#[derive(Debug, Clone, Default)] pub struct AggregationRuntimePlaceholder {}
#[derive(Debug, Clone, Default)] pub struct TriggerRuntimePlaceholder {}
#[derive(Debug, Clone, Default)] pub struct PartitionRuntimePlaceholder {}


#[derive(Debug)]
pub struct SiddhiAppRuntimeBuilder {
    pub siddhi_app_context: Arc<SiddhiAppContext>, // Set upon creation, not Option

    pub stream_definition_map: HashMap<String, Arc<ApiStreamDefinition>>,
    pub table_definition_map: HashMap<String, Arc<crate::query_api::definition::TableDefinition>>,
    pub window_definition_map: HashMap<String, Arc<crate::query_api::definition::WindowDefinition>>,
    pub aggregation_definition_map: HashMap<String, Arc<crate::query_api::definition::AggregationDefinition>>,

    pub stream_junction_map: HashMap<String, Arc<Mutex<StreamJunction>>>,
    pub table_map: HashMap<String, Arc<Mutex<TableRuntimePlaceholder>>>,
    pub window_map: HashMap<String, Arc<Mutex<crate::core::window::WindowRuntime>>>,
    pub aggregation_map: HashMap<String, Arc<Mutex<AggregationRuntimePlaceholder>>>,

    pub query_runtimes: Vec<Arc<QueryRuntime>>,
    pub partition_runtimes: Vec<Arc<PartitionRuntimePlaceholder>>,
    pub trigger_runtimes: Vec<Arc<TriggerRuntimePlaceholder>>,
}

impl SiddhiAppRuntimeBuilder {
    pub fn new(siddhi_app_context: Arc<SiddhiAppContext>) -> Self {
        Self {
            siddhi_app_context, // No longer Option
            stream_definition_map: HashMap::new(),
            table_definition_map: HashMap::new(),
            window_definition_map: HashMap::new(),
            aggregation_definition_map: HashMap::new(),
            stream_junction_map: HashMap::new(),
            table_map: HashMap::new(),
            window_map: HashMap::new(),
            aggregation_map: HashMap::new(),
            query_runtimes: Vec::new(),
            partition_runtimes: Vec::new(),
            trigger_runtimes: Vec::new(),
        }
    }

    // --- Methods to add API definitions ---
    pub fn add_stream_definition(&mut self, stream_def: Arc<ApiStreamDefinition>) {
        self.stream_definition_map.insert(stream_def.abstract_definition.id.clone(), stream_def);
    }
    pub fn add_table_definition(&mut self, table_def: Arc<crate::query_api::definition::TableDefinition>) {
        self.table_definition_map.insert(table_def.abstract_definition.id.clone(), table_def);
    }
    pub fn add_window_definition(&mut self, window_def: Arc<crate::query_api::definition::WindowDefinition>) {
        self.window_definition_map.insert(window_def.stream_definition.abstract_definition.id.clone(), window_def);
    }
     pub fn add_aggregation_definition(&mut self, agg_def: Arc<crate::query_api::definition::AggregationDefinition>) {
        self.aggregation_definition_map.insert(agg_def.abstract_definition.id.clone(), agg_def);
    }
    // FunctionDefinition and TriggerDefinition are handled differently (often registered in context or directly creating runtimes)

    // --- Methods to add runtime components ---
    pub fn add_stream_junction(&mut self, stream_id: String, junction: Arc<Mutex<StreamJunction>>) {
        self.stream_junction_map.insert(stream_id, junction);
    }
    pub fn add_table(&mut self, table_id: String, table_runtime: Arc<Mutex<TableRuntimePlaceholder>>) {
        self.table_map.insert(table_id, table_runtime);
    }
    pub fn add_window(&mut self, window_id: String, window_runtime: Arc<Mutex<crate::core::window::WindowRuntime>>) {
        self.window_map.insert(window_id, window_runtime);
    }
    pub fn add_aggregation_runtime(&mut self, agg_id: String, agg_runtime: Arc<Mutex<AggregationRuntimePlaceholder>>) {
        self.aggregation_map.insert(agg_id, agg_runtime);
    }
    pub fn add_query_runtime(&mut self, query_runtime: Arc<QueryRuntime>) {
        self.query_runtimes.push(query_runtime);
    }
    pub fn add_partition_runtime(&mut self, partition_runtime: Arc<PartitionRuntimePlaceholder>) {
        self.partition_runtimes.push(partition_runtime);
    }
    pub fn add_trigger_runtime(&mut self, trigger_runtime: Arc<TriggerRuntimePlaceholder>) {
        self.trigger_runtimes.push(trigger_runtime);
    }


    // build() method that consumes the builder and returns a SiddhiAppRuntime
    pub fn build(self, api_siddhi_app: Arc<ApiSiddhiApp>) -> Result<SiddhiAppRuntime, String> {
        // Create InputManager and populate it
        let input_manager = crate::core::stream::input::input_manager::InputManager::new(
            Arc::clone(&self.siddhi_app_context),
            self.stream_junction_map.clone(),
        );

        Ok(SiddhiAppRuntime {
            name: self.siddhi_app_context.name.clone(),
            siddhi_app: api_siddhi_app, // The original parsed API definition
            siddhi_app_context: self.siddhi_app_context, // The context created for this app instance
            stream_junction_map: self.stream_junction_map,
            input_manager: Arc::new(input_manager),
            query_runtimes: self.query_runtimes,
            // Initialize other runtime fields (tables, windows, aggregations, partitions, triggers)
            // These would typically be moved from the builder to the runtime instance.
            // For now, they are not fields on SiddhiAppRuntime placeholder.
            // tables: self.table_map,
            // windows: self.window_map,
            // aggregations: self.aggregation_map,
            // partitions: self.partition_runtimes,
            // triggers: self.trigger_runtimes,
            _placeholder_table_map: HashMap::new(), // Placeholder field on SiddhiAppRuntime
        })
    }
}
