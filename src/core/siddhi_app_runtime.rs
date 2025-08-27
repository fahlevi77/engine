// Corresponds to io.siddhi.core.SiddhiAppRuntime (interface)
// and io.siddhi.core.SiddhiAppRuntimeImpl (implementation)

use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::config::siddhi_query_context::SiddhiQueryContext; // For add_callback
use crate::core::config::{ApplicationConfig, SiddhiConfig};
use crate::core::partition::PartitionRuntime;
use crate::core::persistence::SnapshotService;
use crate::core::query::output::callback_processor::CallbackProcessor; // To be created
use crate::core::query::query_runtime::QueryRuntime;
use crate::core::siddhi_app_runtime_builder::TableRuntimePlaceholder;
use crate::core::stream::input::input_handler::InputHandler;
use crate::core::stream::input::input_manager::InputManager;
use crate::core::stream::output::stream_callback::StreamCallback; // The trait
use crate::core::stream::stream_junction::StreamJunction;
use crate::core::trigger::TriggerRuntime;
use crate::core::util::parser::siddhi_app_parser::SiddhiAppParser; // For SiddhiAppParser::parse_siddhi_app_runtime_builder
use crate::core::window::WindowRuntime;
use crate::query_api::SiddhiApp as ApiSiddhiApp; // From query_api
use std::sync::{Arc, Mutex};
use uuid::Uuid;

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
    pub partition_runtimes: Vec<Arc<PartitionRuntime>>,
    pub trigger_runtimes: Vec<Arc<TriggerRuntime>>,
    pub scheduler: Option<Arc<crate::core::util::Scheduler>>,
    pub table_map: HashMap<String, Arc<Mutex<TableRuntimePlaceholder>>>,
    pub window_map: HashMap<String, Arc<Mutex<WindowRuntime>>>,
    pub aggregation_map: HashMap<String, Arc<Mutex<crate::core::aggregation::AggregationRuntime>>>,
}

impl SiddhiAppRuntime {
    // This 'new' function replaces the direct construction and acts more like
    // SiddhiManager.createSiddhiAppRuntime(api_siddhi_app)
    pub fn new(
        api_siddhi_app: Arc<ApiSiddhiApp>,
        // SiddhiContext is needed to initialize SiddhiAppContext if not already done
        siddhi_context: Arc<crate::core::config::siddhi_context::SiddhiContext>,
        siddhi_app_string: Option<String>,
    ) -> Result<Self, String> {
        // 1. Create SiddhiAppContext using @app level annotations when present
        let mut name = api_siddhi_app.name.clone();
        let mut is_playback = false;
        let mut enforce_order = false;
        let mut root_metrics =
            crate::core::config::siddhi_app_context::MetricsLevelPlaceholder::OFF;
        let mut buffer_size = 0i32;
        let mut transport_creation = false;

        if let Some(app_ann) = api_siddhi_app
            .annotations
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case("app"))
        {
            for el in &app_ann.elements {
                match el.key.to_lowercase().as_str() {
                    "name" => name = el.value.clone(),
                    "playback" => is_playback = el.value.eq_ignore_ascii_case("true"),
                    "enforce.order" | "enforceorder" => {
                        enforce_order = el.value.eq_ignore_ascii_case("true")
                    }
                    "statistics" | "stats" => root_metrics = match el.value.to_lowercase().as_str()
                    {
                        "true" | "basic" => {
                            crate::core::config::siddhi_app_context::MetricsLevelPlaceholder::BASIC
                        }
                        "detail" | "detailed" => {
                            crate::core::config::siddhi_app_context::MetricsLevelPlaceholder::DETAIL
                        }
                        _ => crate::core::config::siddhi_app_context::MetricsLevelPlaceholder::OFF,
                    },
                    "buffer_size" | "buffersize" => {
                        if let Ok(sz) = el.value.parse::<i32>() {
                            buffer_size = sz;
                        }
                    }
                    "transport.channel.creation" => {
                        transport_creation = el.value.eq_ignore_ascii_case("true");
                    }
                    _ => {}
                }
            }
        }

        let mut ctx = SiddhiAppContext::new(
            siddhi_context,
            name.clone(),
            Arc::clone(&api_siddhi_app),
            siddhi_app_string.unwrap_or_default(),
        );
        ctx.set_playback(is_playback);
        ctx.set_enforce_order(enforce_order);
        ctx.set_root_metrics_level(root_metrics);
        if buffer_size > 0 {
            ctx.set_buffer_size(buffer_size);
        }
        ctx.set_transport_channel_creation_enabled(transport_creation);
        
        // Initialize ThreadBarrier if enforce_order is enabled or for persistence coordination
        let thread_barrier = Arc::new(crate::core::util::thread_barrier::ThreadBarrier::new());
        ctx.set_thread_barrier(thread_barrier);
        
        let scheduler = if let Some(exec) = ctx.get_scheduled_executor_service() {
            Arc::new(crate::core::util::Scheduler::new(Arc::clone(
                &exec.executor,
            )))
        } else {
            Arc::new(crate::core::util::Scheduler::new(Arc::new(
                crate::core::util::ExecutorService::default(),
            )))
        };
        ctx.set_scheduler(Arc::clone(&scheduler));
        let mut ss = SnapshotService::new(name.clone());
        if let Some(store) = ctx.siddhi_context.get_persistence_store() {
            ss.persistence_store = Some(store);
        }
        let snapshot_service = Arc::new(ss);
        ctx.set_snapshot_service(Arc::clone(&snapshot_service));
        let siddhi_app_context = Arc::new(ctx);

        // 2. Parse the ApiSiddhiApp into a builder
        let builder =
            SiddhiAppParser::parse_siddhi_app_runtime_builder(&api_siddhi_app, siddhi_app_context, None)?;

        // 3. Build the SiddhiAppRuntime from the builder
        builder.build(api_siddhi_app) // Pass the Arc<ApiSiddhiApp> again
    }

    /// Create a new SiddhiAppRuntime with specific application configuration
    pub fn new_with_config(
        api_siddhi_app: Arc<ApiSiddhiApp>,
        siddhi_context: Arc<crate::core::config::siddhi_context::SiddhiContext>,
        siddhi_app_string: Option<String>,
        app_config: Option<ApplicationConfig>,
    ) -> Result<Self, String> {
        // If we have application configuration, apply it before creating the runtime
        if let Some(config) = app_config {
            // Apply global configuration from app config to the runtime
            Self::new_with_applied_config(
                api_siddhi_app,
                siddhi_context,
                siddhi_app_string,
                &config,
            )
        } else {
            // Fall back to standard creation if no config provided
            Self::new(api_siddhi_app, siddhi_context, siddhi_app_string)
        }
    }

    /// Internal method to create runtime with applied configuration
    fn new_with_applied_config(
        api_siddhi_app: Arc<ApiSiddhiApp>,
        siddhi_context: Arc<crate::core::config::siddhi_context::SiddhiContext>,
        siddhi_app_string: Option<String>,
        app_config: &ApplicationConfig,
    ) -> Result<Self, String> {
        // 1. Create SiddhiAppContext using @app level annotations when present
        let mut name = api_siddhi_app.name.clone();
        let mut is_playback = false;
        let mut enforce_order = false;
        let mut root_metrics =
            crate::core::config::siddhi_app_context::MetricsLevelPlaceholder::OFF;
        let mut buffer_size = 0i32;
        let mut transport_creation = false;

        // Apply configuration-based settings from monitoring
        if let Some(ref monitoring) = app_config.monitoring {
            if monitoring.metrics_enabled {
                root_metrics = crate::core::config::siddhi_app_context::MetricsLevelPlaceholder::BASIC;
            }
            // Note: ApplicationConfig::MonitoringConfig doesn't have detailed_metrics
            // This would be a global configuration setting
        }

        // Then apply @app level annotations (which override config)
        if let Some(app_ann) = api_siddhi_app
            .annotations
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case("app"))
        {
            for el in &app_ann.elements {
                match el.key.to_lowercase().as_str() {
                    "name" => name = el.value.clone(),
                    "playback" => is_playback = el.value.eq_ignore_ascii_case("true"),
                    "enforce.order" | "enforceorder" => {
                        enforce_order = el.value.eq_ignore_ascii_case("true")
                    }
                    "statistics" | "stats" => root_metrics = match el.value.to_lowercase().as_str()
                    {
                        "true" | "basic" => {
                            crate::core::config::siddhi_app_context::MetricsLevelPlaceholder::BASIC
                        }
                        "detail" | "detailed" => {
                            crate::core::config::siddhi_app_context::MetricsLevelPlaceholder::DETAIL
                        }
                        _ => crate::core::config::siddhi_app_context::MetricsLevelPlaceholder::OFF,
                    },
                    "buffer_size" | "buffersize" => {
                        if let Ok(sz) = el.value.parse::<i32>() {
                            buffer_size = sz;
                        }
                    }
                    "transport.channel.creation" => {
                        transport_creation = el.value.eq_ignore_ascii_case("true");
                    }
                    _ => {}
                }
            }
        }

        let mut ctx = SiddhiAppContext::new_with_config(
            siddhi_context,
            name.clone(),
            Arc::clone(&api_siddhi_app),
            String::new(), // siddhi_app_string
            Arc::new(SiddhiConfig::default()), // global_config
            Some(app_config.clone()), // app_config
            None, // config_manager
        );
        
        ctx.set_root_metrics_level(root_metrics);
        if buffer_size > 0 {
            ctx.set_buffer_size(buffer_size);
        }
        ctx.set_transport_channel_creation_enabled(transport_creation);
        
        // Apply additional configuration settings to context
        if let Some(ref _error_handling) = app_config.error_handling {
            // Error handling configuration would be applied here
            // Future implementation will configure error handling strategies
        }
        
        // Initialize ThreadBarrier if enforce_order is enabled or for persistence coordination
        let thread_barrier = Arc::new(crate::core::util::ThreadBarrier::new());
        ctx.set_thread_barrier(Arc::clone(&thread_barrier));

        // 2. Create SnapshotService and configure it
        let mut ss = SnapshotService::new(name.clone());
        if let Some(store) = ctx.siddhi_context.get_persistence_store() {
            ss.persistence_store = Some(store);
        }
        let snapshot_service = Arc::new(ss);
        ctx.set_snapshot_service(Arc::clone(&snapshot_service));
        let siddhi_app_context = Arc::new(ctx);

        // 2. Parse the ApiSiddhiApp into a builder with configuration
        let builder =
            SiddhiAppParser::parse_siddhi_app_runtime_builder(&api_siddhi_app, siddhi_app_context, Some(app_config.clone()))?;

        // 3. Build the SiddhiAppRuntime from the builder
        let runtime = builder.build(api_siddhi_app)?; // Pass the Arc<ApiSiddhiApp> again
        
        // 4. Auto-attach sinks from configuration
        runtime.auto_attach_sinks_from_config(app_config)?;
        
        Ok(runtime)
    }

    pub fn get_input_handler(&self, stream_id: &str) -> Option<Arc<Mutex<InputHandler>>> {
        self.input_manager.get_input_handler(stream_id)
    }

    pub fn get_table_input_handler(
        &self,
        table_id: &str,
    ) -> Option<crate::core::stream::input::table_input_handler::TableInputHandler> {
        self.input_manager.get_table_input_handler(table_id)
    }

    pub fn add_callback(
        &self,
        stream_id: &str,
        callback: Box<dyn StreamCallback>,
    ) -> Result<(), String> {
        let output_junction = self
            .stream_junction_map
            .get(stream_id)
            .ok_or_else(|| format!("StreamJunction '{stream_id}' not found to add callback"))?
            .clone();

        let query_name_for_callback = format!(
            "callback_processor_{}_{}",
            stream_id,
            Uuid::new_v4().hyphenated()
        );
        let query_context_for_callback = Arc::new(SiddhiQueryContext::new(
            Arc::clone(&self.siddhi_app_context),
            query_name_for_callback.clone(),
            None, // No specific partition ID for a generic stream callback processor
        ));

        let callback_processor = Arc::new(Mutex::new(CallbackProcessor::new(
            Arc::new(Mutex::new(callback)),
            Arc::clone(&self.siddhi_app_context),
            query_context_for_callback,
            // query_name_for_callback, // query_name is now in query_context
        )));
        output_junction
            .lock()
            .expect("Output StreamJunction Mutex poisoned")
            .subscribe(callback_processor);
        Ok(())
    }

    pub fn start(&self) {
        if self.scheduler.is_some() {
            // placeholder: scheduler is kept alive by self
            println!("Scheduler initialized for SiddhiAppRuntime '{}'", self.name);
        }
        for tr in &self.trigger_runtimes {
            tr.start();
        }
        for pr in &self.partition_runtimes {
            pr.start();
        }
        println!("SiddhiAppRuntime '{}' started", self.name);
    }

    pub fn shutdown(&self) {
        if let Some(scheduler) = &self.scheduler {
            scheduler.shutdown();
        }
        for tr in &self.trigger_runtimes {
            tr.shutdown();
        }
        for pr in &self.partition_runtimes {
            pr.shutdown();
        }
        for qr in &self.query_runtimes {
            qr.flush();
        }
        // Persisted revisions are retained after shutdown for potential restoration
        println!("SiddhiAppRuntime '{}' shutdown", self.name);
    }

    /// Persist the current snapshot using the configured SnapshotService.
    pub fn persist(&self) -> Result<String, String> {
        let service = self
            .siddhi_app_context
            .get_snapshot_service()
            .ok_or("SnapshotService not set")?;
        service.persist()
    }

    /// Capture a snapshot of the current state via the SnapshotService.
    pub fn snapshot(&self) -> Result<Vec<u8>, String> {
        let service = self
            .siddhi_app_context
            .get_snapshot_service()
            .ok_or("SnapshotService not set")?;
        Ok(service.snapshot())
    }

    /// Restore the given snapshot bytes using the SnapshotService.
    pub fn restore(&self, snapshot: &[u8]) -> Result<(), String> {
        let service = self
            .siddhi_app_context
            .get_snapshot_service()
            .ok_or("SnapshotService not set")?;
        service.set_state(snapshot.to_vec());
        Ok(())
    }

    /// Restore the given revision using the SnapshotService.
    pub fn restore_revision(&self, revision: &str) -> Result<(), String> {
        let service = self
            .siddhi_app_context
            .get_snapshot_service()
            .ok_or("SnapshotService not set")?;
        
        // Use ThreadBarrier to coordinate with event processing threads
        if let Some(barrier) = self.siddhi_app_context.get_thread_barrier() {
            // Lock the barrier to prevent new events from entering
            barrier.lock();
            
            // Wait for all active threads to complete their current processing
            while barrier.get_active_threads() > 0 {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            
            // Perform the restoration while event processing is blocked
            let result = service.restore_revision(revision);
            
            // Clear SelectProcessor group states after restoration to ensure fresh aggregator state
            if result.is_ok() {
                self.clear_select_processor_group_states();
            }
            
            // Unlock the barrier to resume normal processing
            barrier.unlock();
            
            result
        } else {
            // No barrier configured, proceed with restoration (may have timing issues)
            service.restore_revision(revision)
        }
    }

    /// Clear group states in all SelectProcessors to ensure fresh state after restoration
    fn clear_select_processor_group_states(&self) {
        for query_runtime in &self.query_runtimes {
            // Try to access the processor chain and find SelectProcessors
            if let Some(ref processor) = query_runtime.processor_chain_head {
                self.clear_processor_chain_group_states(processor);
            }
        }
    }

    /// Recursively clear group states in processor chains
    fn clear_processor_chain_group_states(&self, processor: &Arc<Mutex<dyn crate::core::query::processor::Processor>>) {
        if let Ok(proc) = processor.lock() {
            // Clear group states for this processor (no-op for non-SelectProcessors)
            proc.clear_group_states();
            
            // Recursively check next processors in the chain
            if let Some(ref next) = proc.next_processor() {
                self.clear_processor_chain_group_states(next);
            }
        }
    }

    /// Query an aggregation runtime using optional `within` and `per` clauses.
    pub fn query_aggregation(
        &self,
        agg_id: &str,
        within: Option<crate::query_api::aggregation::Within>,
        per: Option<crate::query_api::aggregation::TimeDuration>,
    ) -> Vec<Vec<crate::core::event::value::AttributeValue>> {
        if let Some(rt) = self.aggregation_map.get(agg_id) {
            rt.lock().unwrap().query(within, per)
        } else {
            Vec::new()
        }
    }
    
    /// Auto-attach sinks from configuration
    fn auto_attach_sinks_from_config(&self, app_config: &ApplicationConfig) -> Result<(), String> {
        use crate::core::stream::output::sink::SinkFactoryRegistry;
        use crate::core::config::ProcessorConfigReader;
        
        // Create a ProcessorConfigReader with the application configuration
        let config_reader = Arc::new(ProcessorConfigReader::new(
            Some(app_config.clone()),
            None, // Global config would be passed here if available
        ));
        
        // Create a sink factory registry
        let registry = SinkFactoryRegistry::new();
        
        // Iterate through all configured streams
        for (stream_name, stream_config) in &app_config.streams {
            // Check if this stream has a sink configuration
            if let Some(ref sink_config) = stream_config.sink {
                // Create the sink using the factory
                let sink = registry.create_sink(
                    sink_config,
                    Some(Arc::clone(&config_reader)),
                    stream_name,
                )?;
                
                // Attach the sink to the stream
                self.add_callback(stream_name, sink)?;
                
                println!("[SiddhiAppRuntime] Auto-attached sink '{}' to stream '{}'", 
                    sink_config.sink_type, stream_name);
            }
        }
        
        Ok(())
    }
}
