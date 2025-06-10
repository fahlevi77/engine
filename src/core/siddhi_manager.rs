// siddhi_rust/src/core/siddhi_manager.rs
use crate::core::config::siddhi_context::SiddhiContext;
use crate::core::siddhi_app_runtime::SiddhiAppRuntime;
use crate::query_api::siddhi_app::SiddhiApp as ApiSiddhiApp;
// SiddhiAppContext is created by SiddhiAppRuntime::new or its internal parser logic
// use crate::core::config::siddhi_app_context::SiddhiAppContext;

// Use query_compiler::parse (which currently returns Err for actual parsing)
use crate::query_compiler::parse as parse_siddhi_ql_string_to_api_app;
use crate::core::executor::ScalarFunctionExecutor; // Added for UDFs
use crate::core::DataSource; // Added for data sources
// Placeholder for actual persistence store trait/type
use crate::core::config::siddhi_context::{PersistenceStorePlaceholder, ConfigManagerPlaceholder, ExtensionClassPlaceholder, DataSourcePlaceholder};


use std::collections::HashMap;
use std::sync::{Arc, Mutex};
// use rand::Rng; // For generating random app names if not specified

/// Main entry point for managing Siddhi application runtimes.
#[derive(Debug)] // Default not derived as SiddhiContext requires specific init.
pub struct SiddhiManager {
    siddhi_context: Arc<SiddhiContext>,
    siddhi_app_runtime_map: Arc<Mutex<HashMap<String, Arc<SiddhiAppRuntime>>>>,
}

impl SiddhiManager {
    pub fn new() -> Self {
        Self {
            siddhi_context: Arc::new(SiddhiContext::new()), // SiddhiContext::new() provides defaults
            siddhi_app_runtime_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn siddhi_context(&self) -> Arc<SiddhiContext> {
        Arc::clone(&self.siddhi_context)
    }

    pub fn create_siddhi_app_runtime_from_string(
        &self,
        siddhi_app_string: &str
    ) -> Result<Arc<SiddhiAppRuntime>, String> {
        // 1. Parse string to ApiSiddhiApp (using query_compiler)
        let api_siddhi_app = parse_siddhi_ql_string_to_api_app(siddhi_app_string)?; // This currently returns Err
        let api_siddhi_app_arc = Arc::new(api_siddhi_app);

        // 2. Create runtime from ApiSiddhiApp
        self.create_siddhi_app_runtime_from_api(api_siddhi_app_arc, Some(siddhi_app_string.to_string()))
    }

    // Added siddhi_app_str_opt for cases where it's available (from string parsing)
    // or not (when an ApiSiddhiApp is provided directly).
    pub fn create_siddhi_app_runtime_from_api(
        &self,
        api_siddhi_app: Arc<ApiSiddhiApp>,
        siddhi_app_str_opt: Option<String>
    ) -> Result<Arc<SiddhiAppRuntime>, String> {

        // Extract app name from annotations or generate one
        // This logic is similar to Java's SiddhiAppParser.
        let app_name = api_siddhi_app.annotations.iter()
            .find(|ann| ann.name == crate::query_api::constants::ANNOTATION_INFO)
            .and_then(|ann| ann.elements.iter().find(|el| el.key == crate::query_api::constants::ANNOTATION_ELEMENT_NAME))
            .map(|el| el.value.clone())
            .unwrap_or_else(|| format!("siddhi_app_{}", uuid::Uuid::new_v4().hyphenated()));

        // Check if app with this name already exists
        if self.siddhi_app_runtime_map.lock().expect("Mutex poisoned").contains_key(&app_name) {
            return Err(format!("SiddhiApp with name '{}' already exists.", app_name));
        }

        // SiddhiAppRuntime::new now handles creation of SiddhiAppContext and parsing
        let runtime = Arc::new(SiddhiAppRuntime::new(
            Arc::clone(&api_siddhi_app),
            Arc::clone(&self.siddhi_context)
            // siddhi_app_str_opt no longer passed here
        )?);

        self.siddhi_app_runtime_map.lock().expect("Mutex poisoned").insert(app_name.clone(), Arc::clone(&runtime)); // Use app_name for map key
        Ok(runtime)
    }

    pub fn get_siddhi_app_runtime(&self, app_name: &str) -> Option<Arc<SiddhiAppRuntime>> {
        self.siddhi_app_runtime_map.lock().expect("Mutex poisoned").get(app_name).map(Arc::clone)
    }

    pub fn get_siddhi_app_runtimes_keys(&self) -> Vec<String> {
       self.siddhi_app_runtime_map.lock().expect("Mutex poisoned").keys().cloned().collect()
    }

    pub fn shutdown_app_runtime(&self, app_name: &str) -> bool {
       if let Some(runtime) = self.siddhi_app_runtime_map.lock().expect("Mutex poisoned").remove(app_name) {
           runtime.shutdown();
           true
       } else {
           false
       }
    }

    pub fn shutdown_all_app_runtimes(&self) {
        let mut map = self.siddhi_app_runtime_map.lock().expect("Mutex poisoned");
        for runtime in map.values() {
            runtime.shutdown();
        }
        map.clear();
    }

    // --- Setters for SiddhiContext components (placeholders) ---
    // `set_extension` typically refers to general extensions (FunctionExecutor, StreamProcessor etc.)
    // For scalar functions, we use `add_scalar_function_factory`.
    pub fn set_extension(&self, _name: &str, _extension_class_placeholder: ExtensionClassPlaceholder) -> Result<(), String> {
        // This would typically involve:
        // 1. Loading the class/factory based on `extension_class_placeholder`.
        // 2. Determining its type (Function, StreamProcessor, Window, etc.).
        // 3. Registering it with the appropriate manager or map in SiddhiContext (e.g., for StreamProcessors).
        // For ScalarFunctionExecutor, `add_scalar_function_factory` is more specific.
        println!("[SiddhiManager] set_extension called for {} (General Placeholder - use add_scalar_function_factory for UDFs)", _name);
        Err("General set_extension not fully implemented; use specific adders like add_scalar_function_factory.".to_string())
    }

    // Specific method for adding scalar function factories
    pub fn add_scalar_function_factory(&self, name: String, function_factory: Box<dyn ScalarFunctionExecutor>) {
        self.siddhi_context.add_scalar_function_factory(name, function_factory);
    }

    pub fn add_window_factory(&self, name: String, factory: Box<dyn crate::core::extension::WindowProcessorFactory>) {
        self.siddhi_context.add_window_factory(name, factory);
    }

    pub fn add_attribute_aggregator_factory(&self, name: String, factory: Box<dyn crate::core::extension::AttributeAggregatorFactory>) {
        self.siddhi_context.add_attribute_aggregator_factory(name, factory);
    }

    pub fn add_source_factory(&self, name: String, factory: Box<dyn crate::core::extension::SourceFactory>) {
        self.siddhi_context.add_source_factory(name, factory);
    }

    pub fn add_sink_factory(&self, name: String, factory: Box<dyn crate::core::extension::SinkFactory>) {
        self.siddhi_context.add_sink_factory(name, factory);
    }

    pub fn add_store_factory(&self, name: String, factory: Box<dyn crate::core::extension::StoreFactory>) {
        self.siddhi_context.add_store_factory(name, factory);
    }

    pub fn add_source_mapper_factory(&self, name: String, factory: Box<dyn crate::core::extension::SourceMapperFactory>) {
        self.siddhi_context.add_source_mapper_factory(name, factory);
    }

    pub fn add_sink_mapper_factory(&self, name: String, factory: Box<dyn crate::core::extension::SinkMapperFactory>) {
        self.siddhi_context.add_sink_mapper_factory(name, factory);
    }

    // Specific method for adding data sources
    pub fn add_data_source(&self, name: String, data_source: Arc<dyn DataSource>) {
        self.siddhi_context.add_data_source(name, data_source);
    }

    // set_data_source was a placeholder, replaced by add_data_source
    // pub fn set_data_source(&self, name: &str, ds_placeholder: DataSourcePlaceholder) -> Result<(), String> { ... }


    pub fn set_persistence_store(&self, _ps_placeholder: PersistenceStorePlaceholder) -> Result<(), String> {
        // self.siddhi_context.set_persistence_store(ps_placeholder);
        println!("[SiddhiManager] set_persistence_store called (Placeholder)");
        Ok(())
    }

    pub fn set_config_manager(&self, _cm_placeholder: ConfigManagerPlaceholder) -> Result<(), String> {
       println!("[SiddhiManager] set_config_manager called (Placeholder)");
       Ok(())
    }

    // --- Validation ---
    pub fn validate_siddhi_app_string(&self, siddhi_app_string: &str) -> Result<(), String> {
        let api_siddhi_app = parse_siddhi_ql_string_to_api_app(siddhi_app_string)?;
        self.validate_siddhi_app_api(&Arc::new(api_siddhi_app), Some(siddhi_app_string.to_string()))
    }

    pub fn validate_siddhi_app_api(&self, api_siddhi_app: &Arc<ApiSiddhiApp>, siddhi_app_str_opt: Option<String>) -> Result<(), String> {
        // Create a temporary runtime to validate
        // Note: This doesn't add it to the main siddhi_app_runtime_map
        let temp_runtime = SiddhiAppRuntime::new(
            Arc::clone(api_siddhi_app),
            Arc::clone(&self.siddhi_context)
            // siddhi_app_str_opt no longer passed here
        )?;
        temp_runtime.start(); // This would internally build the plan, run initializations
        temp_runtime.shutdown(); // Clean up
        Ok(())
    }

    // --- Persistence ---
    pub fn persist_all(&self) {
        for runtime in self.siddhi_app_runtime_map.lock().expect("Mutex poisoned").values() {
            // runtime.persist(); // SiddhiAppRuntime needs persist() method
        }
        println!("[SiddhiManager] persist_all called (Placeholder)");
    }

    pub fn restore_all_last_state(&self) {
        for runtime in self.siddhi_app_runtime_map.lock().expect("Mutex poisoned").values() {
            // runtime.restore_last_revision(); // SiddhiAppRuntime needs this
        }
        println!("[SiddhiManager] restore_all_last_state called (Placeholder)");
    }
}

impl Default for SiddhiManager {
    fn default() -> Self {
        Self::new()
    }
}

// Helper on ApiSiddhiApp for getting name (if not already part of query_api)
// This is more robust than assuming api_siddhi_app.name
impl ApiSiddhiApp {
    fn get_name(&self) -> Option<String> {
        self.annotations.iter()
            .find(|ann| ann.name == crate::query_api::constants::ANNOTATION_INFO)
            .and_then(|ann| ann.elements.iter().find(|el| el.key == crate::query_api::constants::ANNOTATION_ELEMENT_NAME))
            .map(|el| el.value.clone())
    }
}
