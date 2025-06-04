// Corresponds to io.siddhi.core.SiddhiAppRuntime (interface)
// and io.siddhi.core.SiddhiAppRuntimeImpl (implementation)

use crate::query_api::SiddhiApp as ApiSiddhiApp; // From query_api
use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::config::siddhi_query_context::SiddhiQueryContext; // For add_callback
use crate::core::stream::stream_junction::StreamJunction;
use crate::core::stream::input::input_manager::InputManager;
use crate::core::stream::input::input_handler::InputHandler;
use std::sync::{Arc, Mutex};
use crate::core::stream::output::stream_callback::StreamCallback; // The trait
use crate::core::query::query_runtime::QueryRuntime;
use crate::core::util::parser::SiddhiAppParser; // For SiddhiAppParser::parse_siddhi_app_runtime_builder
use crate::core::siddhi_app_runtime_builder::SiddhiAppRuntimeBuilder;
use crate::core::query::output::callback_processor::CallbackProcessor; // To be created
use crate::core::query::processor::Processor; // Trait for CallbackProcessor

use std::collections::HashMap;

/// Manages the runtime lifecycle of a single Siddhi Application.
#[derive(Debug)] // Default removed, construction via new() -> Result
pub struct SiddhiAppRuntime {
    pub name: String,
    pub siddhi_app: Arc<ApiSiddhiApp>, // The original parsed API definition
    pub siddhi_app_context: Arc<SiddhiAppContext>,

    // Runtime components constructed by SiddhiAppRuntimeBuilder
    pub stream_junction_map: HashMap<String, Arc<Mutex<StreamJunction>>>,
    pub input_manager: Arc<InputManager>,
    pub query_runtimes: Vec<Arc<QueryRuntime>>,
    // TODO: Add other runtime component maps (tables, windows, aggregations, partitions, triggers)
    // These would be moved from SiddhiAppRuntimeBuilder during the build() process.
    // For now, using a placeholder to acknowledge they would exist.
    pub _placeholder_table_map: HashMap<String, String>, // Example placeholder
}

impl SiddhiAppRuntime {
    // This 'new' function replaces the direct construction and acts more like
    // SiddhiManager.createSiddhiAppRuntime(api_siddhi_app)
    pub fn new(
        api_siddhi_app: Arc<ApiSiddhiApp>,
        // SiddhiContext is needed to initialize SiddhiAppContext if not already done
        siddhi_context: Arc<crate::core::config::siddhi_context::SiddhiContext>
    ) -> Result<Self, String> {

        // 1. Create SiddhiAppContext (mimicking part of SiddhiAppParser.parse)
        // In a full setup, SiddhiAppParser.parse would create this context first.
        // For now, assuming a basic AppContext can be created here for the builder.
        let app_name = api_siddhi_app.annotations.iter()
            .find(|ann| ann.name == "info") // Assuming query_api::Annotation
            .and_then(|ann| ann.elements.iter().find(|el| el.key == "name"))
            .map(|el| el.value.clone())
            .unwrap_or_else(|| format!("siddhi_app_{}", uuid::Uuid::new_v4().hyphenated()));

        // TODO: Populate SiddhiAppContext more fully from @app annotations and SiddhiContext
        let siddhi_app_context = Arc::new(SiddhiAppContext::new(
            siddhi_context,
            app_name,
            Arc::clone(&api_siddhi_app),
            String::new() // siddhi_app_string - can be empty if already parsed
        ));

        // 2. Parse the ApiSiddhiApp into a builder
        let builder = SiddhiAppParser::parse_siddhi_app_runtime_builder(&api_siddhi_app, siddhi_app_context)?;

        // 3. Build the SiddhiAppRuntime from the builder
        builder.build(api_siddhi_app) // Pass the Arc<ApiSiddhiApp> again
    }

    pub fn get_input_handler(&self, stream_id: &str) -> Option<Arc<Mutex<InputHandler>>> {
        self.input_manager.get_input_handler(stream_id)
    }

    pub fn add_callback(&self, stream_id: &str, callback: Box<dyn StreamCallback>) -> Result<(), String> {
        let output_junction = self.stream_junction_map.get(stream_id)
            .ok_or_else(|| format!("StreamJunction '{}' not found to add callback", stream_id))?
            .clone();

        let query_name_for_callback = format!("callback_processor_{}_{}", stream_id, uuid::Uuid::new_v4().hyphenated());
        let query_context_for_callback = Arc::new(
           SiddhiQueryContext::new(
               Arc::clone(&self.siddhi_app_context),
               query_name_for_callback.clone(),
               None // No specific partition ID for a generic stream callback processor
           ));

        let callback_processor = Arc::new(Mutex::new(CallbackProcessor::new(
            Arc::new(Mutex::new(callback)),
            Arc::clone(&self.siddhi_app_context),
            query_context_for_callback,
            // query_name_for_callback, // query_name is now in query_context
        )));
        output_junction.lock().expect("Output StreamJunction Mutex poisoned").add_subscriber(callback_processor);
        Ok(())
    }

    pub fn start(&self) {
        // TODO: Call start on StreamJunctions (if they have async tasks)
        // TODO: Call start on Schedulers for Triggers
        // TODO: Call start on Sources (part of InputManager or StreamJunctions with sources)
        println!("SiddhiAppRuntime '{}' started (Placeholder)", self.name);
    }

    pub fn shutdown(&self) {
        // TODO: Call stop on StreamJunctions, Schedulers, Sources, close services in AppContext
        println!("SiddhiAppRuntime '{}' shutdown (Placeholder)", self.name);
    }

    // TODO: Implement other methods from SiddhiAppRuntime interface:
    // get_stream_definition_map, get_table_definition_map, etc. (from composed ApiSiddhiApp)
    // get_sources, get_sinks, get_tables (runtime instances), get_queries, get_partitions
    // query(onDemandQueryString), persist, snapshot, restore, etc.
}
