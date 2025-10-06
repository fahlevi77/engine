// siddhi_rust/src/core/siddhi_manager.rs
use crate::core::config::siddhi_context::SiddhiContext;
use crate::core::siddhi_app_runtime::SiddhiAppRuntime;
use crate::query_api::siddhi_app::SiddhiApp as ApiSiddhiApp;
// SiddhiAppContext is created by SiddhiAppRuntime::new or its internal parser logic
// use crate::core::config::siddhi_app_context::SiddhiAppContext;

// Use SQL compiler for parsing (sqlparser-rs based)
use crate::core::executor::ScalarFunctionExecutor; // Added for UDFs
use crate::core::DataSource;
                                                                       // Placeholder for actual persistence store trait/type
use crate::core::config::{ConfigManager, SiddhiConfig, ApplicationConfig};
use crate::core::config::siddhi_context::ExtensionClassPlaceholder;
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
    config_manager: Option<Arc<ConfigManager>>,
    cached_config: Arc<Mutex<Option<SiddhiConfig>>>,
}

impl SiddhiManager {
    pub fn new() -> Self {
        Self {
            siddhi_context: Arc::new(SiddhiContext::new()), // SiddhiContext::new() provides defaults
            siddhi_app_runtime_map: Arc::new(Mutex::new(HashMap::new())),
            loaded_libraries: Arc::new(Mutex::new(Vec::new())),
            config_manager: None,
            cached_config: Arc::new(Mutex::new(None)),
        }
    }

    /// Create SiddhiManager with a specific configuration
    pub fn new_with_config(config: SiddhiConfig) -> Self {
        let manager = Self::new();
        *manager.cached_config.lock().expect("Mutex poisoned") = Some(config);
        manager
    }

    /// Create SiddhiManager with ConfigManager for dynamic configuration loading
    pub fn new_with_config_manager(config_manager: ConfigManager) -> Self {
        Self {
            siddhi_context: Arc::new(SiddhiContext::new()),
            siddhi_app_runtime_map: Arc::new(Mutex::new(HashMap::new())),
            loaded_libraries: Arc::new(Mutex::new(Vec::new())),
            config_manager: Some(Arc::new(config_manager)),
            cached_config: Arc::new(Mutex::new(None)),
        }
    }

    pub fn siddhi_context(&self) -> Arc<SiddhiContext> {
        Arc::clone(&self.siddhi_context)
    }

    pub async fn create_siddhi_app_runtime_from_string(
        &self,
        siddhi_app_string: &str,
    ) -> Result<Arc<SiddhiAppRuntime>, String> {
        // 1. Parse SQL string to SqlApplication (using sql_compiler)
        let sql_app = crate::sql_compiler::parse_sql_application(siddhi_app_string)
            .map_err(|e| format!("SQL parse error: {}", e))?;

        // 2. Convert to SiddhiApp
        let app_name = sql_app.catalog.get_stream_names()
            .first()
            .map(|s| format!("App_{}", s))
            .unwrap_or_else(|| "SiddhiApp".to_string());
        let api_siddhi_app = sql_app.to_siddhi_app(app_name);
        let api_siddhi_app_arc = Arc::new(api_siddhi_app);

        // 3. Create runtime from ApiSiddhiApp
        self.create_siddhi_app_runtime_from_api(
            api_siddhi_app_arc,
            Some(siddhi_app_string.to_string()),
        )
        .await
    }

    // Added siddhi_app_str_opt for cases where it's available (from string parsing)
    // or not (when an ApiSiddhiApp is provided directly).
    pub async fn create_siddhi_app_runtime_from_api(
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

        // Get application-specific configuration
        let app_config = self.get_application_config(&app_name).await?;
        
        // SiddhiAppRuntime::new now handles creation of SiddhiAppContext and parsing
        let runtime = Arc::new(SiddhiAppRuntime::new_with_config(
            Arc::clone(&api_siddhi_app),
            Arc::clone(&self.siddhi_context),
            siddhi_app_str_opt.clone(),
            app_config,
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

    /// Set the configuration manager for dynamic configuration loading
    pub fn set_config_manager(&mut self, config_manager: ConfigManager) -> Result<(), String> {
        self.config_manager = Some(Arc::new(config_manager));
        // Clear cached config to force reload on next access
        *self.cached_config.lock().map_err(|e| format!("Mutex poisoned: {}", e))? = None;
        Ok(())
    }

    /// Get the current configuration, loading it if necessary
    pub async fn get_config(&self) -> Result<SiddhiConfig, String> {
        // Check if we have a cached config
        {
            let cached = self.cached_config.lock().map_err(|e| format!("Mutex poisoned: {}", e))?;
            if let Some(ref config) = *cached {
                return Ok(config.clone());
            }
        }

        // Load configuration using ConfigManager if available
        if let Some(ref config_manager) = self.config_manager {
            let config = config_manager
                .load_unified_config()
                .await
                .map_err(|e| format!("Failed to load configuration: {}", e))?;
            
            // Cache the loaded configuration
            *self.cached_config.lock().map_err(|e| format!("Mutex poisoned: {}", e))? = Some(config.clone());
            
            Ok(config)
        } else {
            // Return default configuration if no ConfigManager is set
            let default_config = SiddhiConfig::default();
            *self.cached_config.lock().map_err(|e| format!("Mutex poisoned: {}", e))? = Some(default_config.clone());
            Ok(default_config)
        }
    }

    /// Get application-specific configuration by name
    pub async fn get_application_config(&self, app_name: &str) -> Result<Option<ApplicationConfig>, String> {
        let config = self.get_config().await?;
        Ok(config.applications.get(app_name).cloned())
    }

    // --- Validation ---
    pub fn validate_siddhi_app_string(&self, siddhi_app_string: &str) -> Result<(), String> {
        // Parse SQL string to SqlApplication
        let sql_app = crate::sql_compiler::parse_sql_application(siddhi_app_string)
            .map_err(|e| format!("SQL parse error: {}", e))?;

        // Convert to SiddhiApp
        let app_name = sql_app.catalog.get_stream_names()
            .first()
            .map(|s| format!("App_{}", s))
            .unwrap_or_else(|| "SiddhiApp".to_string());
        let api_siddhi_app = sql_app.to_siddhi_app(app_name);

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

    /// Level 3 API: Create SiddhiAppRuntime from SQL application string
    ///
    /// This is the high-level API for creating a runtime directly from SQL.
    /// Internally uses the SQL compiler to parse and convert to SiddhiApp.
    ///
    /// # Example
    /// ```rust,ignore
    /// let manager = SiddhiManager::new();
    /// let sql = r#"
    ///     CREATE STREAM StockStream (symbol VARCHAR, price DOUBLE);
    ///
    ///     SELECT symbol, price
    ///     FROM StockStream
    ///     WHERE price > 100;
    /// "#;
    ///
    /// let runtime = manager.create_runtime_from_sql(sql, Some("MyApp".to_string())).await?;
    /// runtime.start();
    /// ```
    pub async fn create_runtime_from_sql(
        &self,
        sql: &str,
        app_name: Option<String>,
    ) -> Result<Arc<SiddhiAppRuntime>, String> {
        // Parse SQL application
        let sql_app = crate::sql_compiler::parse_sql_application(sql)
            .map_err(|e| format!("SQL parse error: {}", e))?;

        // Convert to SiddhiApp
        let app_name = app_name.unwrap_or_else(|| "SqlApp".to_string());
        let siddhi_app = sql_app.to_siddhi_app(app_name);

        // Create runtime using existing method
        self.create_siddhi_app_runtime_from_api(Arc::new(siddhi_app), Some(sql.to_string()))
            .await
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
