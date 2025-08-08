// siddhi_rust/src/core/siddhi_manager.rs
use crate::core::config::siddhi_context::SiddhiContext;
use crate::core::siddhi_app_runtime::SiddhiAppRuntime;
use crate::query_api::siddhi_app::SiddhiApp as ApiSiddhiApp;
// SiddhiAppContext is created by SiddhiAppRuntime::new or its internal parser logic
// use crate::core::config::siddhi_app_context::SiddhiAppContext;

// Use query_compiler::parse (which currently returns Err for actual parsing)
use crate::core::executor::ScalarFunctionExecutor; // Added for UDFs
use crate::core::DataSource;
use crate::query_compiler::parse as parse_siddhi_ql_string_to_api_app; // Added for data sources
                                                                       // Placeholder for actual persistence store trait/type
use crate::core::config::siddhi_context::{ConfigManagerPlaceholder, ExtensionClassPlaceholder};
use crate::core::extension;
use crate::core::persistence::PersistenceStore;

use libloading::Library;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
// use rand::Rng; // For generating random app names if not specified

/// Main entry point for managing Siddhi application runtimes.
#[repr(C)]
pub struct SiddhiManager {
    siddhi_context: Arc<SiddhiContext>,
    siddhi_app_runtime_map: Arc<Mutex<HashMap<String, Arc<SiddhiAppRuntime>>>>,
    loaded_libraries: Arc<Mutex<Vec<std::mem::ManuallyDrop<libloading::Library>>>>,
}

impl SiddhiManager {
    pub fn new() -> Self {
        Self {
            siddhi_context: Arc::new(SiddhiContext::new()), // SiddhiContext::new() provides defaults
            siddhi_app_runtime_map: Arc::new(Mutex::new(HashMap::new())),
            loaded_libraries: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn siddhi_context(&self) -> Arc<SiddhiContext> {
        Arc::clone(&self.siddhi_context)
    }

    pub fn create_siddhi_app_runtime_from_string(
        &self,
        siddhi_app_string: &str,
    ) -> Result<Arc<SiddhiAppRuntime>, String> {
        // 1. Parse string to ApiSiddhiApp (using query_compiler)
        let api_siddhi_app = parse_siddhi_ql_string_to_api_app(siddhi_app_string)?; // This currently returns Err
        let api_siddhi_app_arc = Arc::new(api_siddhi_app);

        // 2. Create runtime from ApiSiddhiApp
        self.create_siddhi_app_runtime_from_api(
            api_siddhi_app_arc,
            Some(siddhi_app_string.to_string()),
        )
    }

    // Added siddhi_app_str_opt for cases where it's available (from string parsing)
    // or not (when an ApiSiddhiApp is provided directly).
    pub fn create_siddhi_app_runtime_from_api(
        &self,
        api_siddhi_app: Arc<ApiSiddhiApp>,
        siddhi_app_str_opt: Option<String>,
    ) -> Result<Arc<SiddhiAppRuntime>, String> {
        // Extract app name from annotations or generate one
        // This logic is similar to Java's SiddhiAppParser.
        let app_name = api_siddhi_app
            .annotations
            .iter()
            .find(|ann| ann.name == crate::query_api::constants::ANNOTATION_INFO)
            .and_then(|ann| {
                ann.elements
                    .iter()
                    .find(|el| el.key == crate::query_api::constants::ANNOTATION_ELEMENT_NAME)
            })
            .map(|el| el.value.clone())
            .unwrap_or_else(|| format!("siddhi_app_{}", uuid::Uuid::new_v4().hyphenated()));

        // Check if app with this name already exists
        if self
            .siddhi_app_runtime_map
            .lock()
            .expect("Mutex poisoned")
            .contains_key(&app_name)
        {
            return Err(format!(
                "SiddhiApp with name '{app_name}' already exists."
            ));
        }

        // SiddhiAppRuntime::new now handles creation of SiddhiAppContext and parsing
        let runtime = Arc::new(SiddhiAppRuntime::new(
            Arc::clone(&api_siddhi_app),
            Arc::clone(&self.siddhi_context),
            siddhi_app_str_opt.clone(),
        )?);
        runtime.start();

        self.siddhi_app_runtime_map
            .lock()
            .expect("Mutex poisoned")
            .insert(app_name.clone(), Arc::clone(&runtime)); // Use app_name for map key
        Ok(runtime)
    }

    pub fn get_siddhi_app_runtime(&self, app_name: &str) -> Option<Arc<SiddhiAppRuntime>> {
        self.siddhi_app_runtime_map
            .lock()
            .expect("Mutex poisoned")
            .get(app_name)
            .map(Arc::clone)
    }

    pub fn get_siddhi_app_runtimes_keys(&self) -> Vec<String> {
        self.siddhi_app_runtime_map
            .lock()
            .expect("Mutex poisoned")
            .keys()
            .cloned()
            .collect()
    }

    pub fn shutdown_app_runtime(&self, app_name: &str) -> bool {
        if let Some(runtime) = self
            .siddhi_app_runtime_map
            .lock()
            .expect("Mutex poisoned")
            .remove(app_name)
        {
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
    pub fn set_extension(
        &self,
        name: &str,
        library_path: ExtensionClassPlaceholder,
    ) -> Result<(), String> {
        unsafe {
            let lib = Library::new(&library_path).map_err(|e| e.to_string())?;

            macro_rules! call_if_exists {
                ($sym:expr) => {
                    if let Ok(f) = lib.get::<extension::RegisterFn>($sym) {
                        f(self);
                    }
                };
            }

            call_if_exists!(extension::REGISTER_EXTENSION_FN);
            call_if_exists!(extension::REGISTER_WINDOWS_FN);
            call_if_exists!(extension::REGISTER_FUNCTIONS_FN);
            call_if_exists!(extension::REGISTER_SOURCES_FN);
            call_if_exists!(extension::REGISTER_SINKS_FN);
            call_if_exists!(extension::REGISTER_STORES_FN);
            call_if_exists!(extension::REGISTER_SOURCE_MAPPERS_FN);
            call_if_exists!(extension::REGISTER_SINK_MAPPERS_FN);

            self.loaded_libraries
                .lock()
                .unwrap()
                .push(std::mem::ManuallyDrop::new(lib));
        }

        println!(
            "[SiddhiManager] dynamically loaded extension '{name}' from {library_path}"
        );
        Ok(())
    }

    // Specific method for adding scalar function factories
    pub fn add_scalar_function_factory(
        &self,
        name: String,
        function_factory: Box<dyn ScalarFunctionExecutor>,
    ) {
        self.siddhi_context
            .add_scalar_function_factory(name, function_factory);
    }

    pub fn add_window_factory(
        &self,
        name: String,
        factory: Box<dyn crate::core::extension::WindowProcessorFactory>,
    ) {
        self.siddhi_context.add_window_factory(name, factory);
    }

    pub fn add_attribute_aggregator_factory(
        &self,
        name: String,
        factory: Box<dyn crate::core::extension::AttributeAggregatorFactory>,
    ) {
        self.siddhi_context
            .add_attribute_aggregator_factory(name, factory);
    }

    pub fn add_source_factory(
        &self,
        name: String,
        factory: Box<dyn crate::core::extension::SourceFactory>,
    ) {
        self.siddhi_context.add_source_factory(name, factory);
    }

    pub fn add_sink_factory(
        &self,
        name: String,
        factory: Box<dyn crate::core::extension::SinkFactory>,
    ) {
        self.siddhi_context.add_sink_factory(name, factory);
    }

    pub fn add_store_factory(
        &self,
        name: String,
        factory: Box<dyn crate::core::extension::StoreFactory>,
    ) {
        self.siddhi_context.add_store_factory(name, factory);
    }

    pub fn add_source_mapper_factory(
        &self,
        name: String,
        factory: Box<dyn crate::core::extension::SourceMapperFactory>,
    ) {
        self.siddhi_context.add_source_mapper_factory(name, factory);
    }

    pub fn add_sink_mapper_factory(
        &self,
        name: String,
        factory: Box<dyn crate::core::extension::SinkMapperFactory>,
    ) {
        self.siddhi_context.add_sink_mapper_factory(name, factory);
    }

    pub fn add_table_factory(
        &self,
        name: String,
        factory: Box<dyn crate::core::extension::TableFactory>,
    ) {
        self.siddhi_context.add_table_factory(name, factory);
    }

    // Specific method for adding data sources
    pub fn add_data_source(
        &self,
        name: String,
        data_source: Arc<dyn DataSource>,
    ) -> Result<(), String> {
        self.siddhi_context.add_data_source(name, data_source)
    }

    // set_data_source was a placeholder, replaced by add_data_source
    // pub fn set_data_source(&self, name: &str, ds_placeholder: DataSourcePlaceholder) -> Result<(), String> { ... }

    pub fn set_persistence_store(&self, store: Arc<dyn PersistenceStore>) {
        self.siddhi_context.set_persistence_store(store);
    }

    pub fn set_config_manager(
        &self,
        _cm_placeholder: ConfigManagerPlaceholder,
    ) -> Result<(), String> {
        println!("[SiddhiManager] set_config_manager called (Placeholder)");
        Ok(())
    }

    // --- Validation ---
    pub fn validate_siddhi_app_string(&self, siddhi_app_string: &str) -> Result<(), String> {
        let api_siddhi_app = parse_siddhi_ql_string_to_api_app(siddhi_app_string)?;
        self.validate_siddhi_app_api(
            &Arc::new(api_siddhi_app),
            Some(siddhi_app_string.to_string()),
        )
    }

    pub fn validate_siddhi_app_api(
        &self,
        api_siddhi_app: &Arc<ApiSiddhiApp>,
        siddhi_app_str_opt: Option<String>,
    ) -> Result<(), String> {
        // Create a temporary runtime to validate
        // Note: This doesn't add it to the main siddhi_app_runtime_map
        let temp_runtime = SiddhiAppRuntime::new(
            Arc::clone(api_siddhi_app),
            Arc::clone(&self.siddhi_context),
            siddhi_app_str_opt,
        )?;
        temp_runtime.start(); // This would internally build the plan, run initializations
        temp_runtime.shutdown(); // Clean up
        Ok(())
    }

    // --- Persistence ---
    pub fn persist_app(&self, app_name: &str) -> Result<String, String> {
        let map = self.siddhi_app_runtime_map.lock().expect("Mutex poisoned");
        let rt = map
            .get(app_name)
            .ok_or_else(|| format!("SiddhiApp '{app_name}' not found"))?;
        rt.persist()
    }

    pub fn restore_app_revision(&self, app_name: &str, revision: &str) -> Result<(), String> {
        let map = self.siddhi_app_runtime_map.lock().expect("Mutex poisoned");
        let rt = map
            .get(app_name)
            .ok_or_else(|| format!("SiddhiApp '{app_name}' not found"))?;
        rt.restore_revision(revision)
    }

    pub fn persist_all(&self) {
        for runtime in self
            .siddhi_app_runtime_map
            .lock()
            .expect("Mutex poisoned")
            .values()
        {
            let _ = runtime.persist();
        }
    }

    pub fn restore_all_last_state(&self) {
        for runtime in self
            .siddhi_app_runtime_map
            .lock()
            .expect("Mutex poisoned")
            .values()
        {
            if let Some(service) = runtime.siddhi_app_context.get_snapshot_service() {
                if let Some(store) = &service.persistence_store {
                    if let Some(last) = store.get_last_revision(&runtime.name) {
                        let _ = runtime.restore_revision(&last);
                    }
                }
            }
        }
    }
}

impl Default for SiddhiManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SiddhiManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SiddhiManager")
            .field("siddhi_context", &"SiddhiContext")
            .field(
                "app_runtimes",
                &self.siddhi_app_runtime_map.lock().unwrap().len(),
            )
            .finish()
    }
}

// Helper on ApiSiddhiApp for getting name (if not already part of query_api)
// This is more robust than assuming api_siddhi_app.name
impl ApiSiddhiApp {
    fn get_name(&self) -> Option<String> {
        self.annotations
            .iter()
            .find(|ann| ann.name == crate::query_api::constants::ANNOTATION_INFO)
            .and_then(|ann| {
                ann.elements
                    .iter()
                    .find(|el| el.key == crate::query_api::constants::ANNOTATION_ELEMENT_NAME)
            })
            .map(|el| el.value.clone())
    }
}
