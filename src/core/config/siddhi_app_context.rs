// Corresponds to io.siddhi.core.config.SiddhiAppContext
use super::siddhi_context::SiddhiContext;
use crate::core::util::executor_service::ExecutorService;
use crate::core::util::id_generator::IdGenerator;
use crate::core::util::Scheduler;
use crate::query_api::SiddhiApp; // From query_api
use std::cell::RefCell;
use std::collections::HashMap; // For scriptFunctionMap
use std::sync::Arc;
use std::thread_local;
// use super::statistics_manager::StatisticsManager; // TODO: Define later
// use super::timestamp_generator::TimestampGenerator; // TODO: Define later
// use super::snapshot_service::SnapshotService; // TODO: Define later
// use super::id_generator::IdGenerator; // TODO: Define later
// use crate::core::function::Script; // TODO: Define later
// use crate::core::trigger::Trigger; // TODO: Define later
// use crate::core::util::extension::ExternalReferencedHolder; // TODO: Define later
// use crate::core::util::Scheduler; // TODO: Define later
// use com::lmax::disruptor::ExceptionHandler; // This is a Java type
// use java::beans::ExceptionListener; // This is a Java type

// Placeholders for complex Java/Siddhi types not yet ported/defined
#[derive(Debug, Clone, Default)]
pub struct StatisticsManagerPlaceholder {}
#[derive(Debug, Clone, Default)]
pub struct TimestampGeneratorPlaceholder {}
use crate::core::persistence::SnapshotService;
use crate::core::util::thread_barrier::ThreadBarrier;
#[derive(Debug, Clone, Default)]
pub struct ScriptPlaceholder {}
#[derive(Debug, Clone)]
pub struct ScriptFunctionPlaceholder {
    pub return_type: crate::query_api::definition::attribute::Type,
}

impl Default for ScriptFunctionPlaceholder {
    fn default() -> Self {
        Self {
            return_type: crate::query_api::definition::attribute::Type::OBJECT,
        }
    }
}
#[derive(Debug, Clone, Default)]
pub struct TriggerPlaceholder {}
#[derive(Debug, Clone, Default)]
pub struct ExternalReferencedHolderPlaceholder {}
#[derive(Debug, Clone, Default)]
pub struct SchedulerPlaceholder {}
#[derive(Debug, Clone, Default)]
pub struct DisruptorExceptionHandlerPlaceholder {}
#[derive(Debug, Clone, Default)]
pub struct ExceptionListenerPlaceholder {}

thread_local! {
    static GROUP_BY_KEY: RefCell<Option<String>> = RefCell::new(None);
    static PARTITION_KEY: RefCell<Option<String>> = RefCell::new(None);
}

// Placeholder for stats Level enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MetricsLevelPlaceholder {
    #[default]
    OFF,
    BASIC,
    DETAIL,
}

/// Context specific to a single Siddhi Application instance.
#[derive(Debug, Clone)] // Default is tricky due to Arc<SiddhiApp> and Arc<SiddhiContext>
pub struct SiddhiAppContext {
    pub siddhi_context: Arc<SiddhiContext>, // Shared global context
    pub name: String,                       // Name of the SiddhiApp
    pub siddhi_app_string: String,          // The raw SiddhiQL string
    pub siddhi_app: Arc<SiddhiApp>,         // Parsed SiddhiApp definition from query_api

    pub is_playback: bool,
    pub is_enforce_order: bool,
    pub root_metrics_level: MetricsLevelPlaceholder,
    pub statistics_manager: Option<StatisticsManagerPlaceholder>, // Manages collection and reporting of runtime statistics. Option because it's set post-construction in Java.

    // Threading and scheduling (using placeholders)
    pub executor_service: Option<Arc<ExecutorService>>, // General purpose thread pool for the Siddhi App.
    pub scheduled_executor_service: Option<Arc<crate::core::util::ScheduledExecutorService>>, // Thread pool for scheduled tasks.
    pub scheduler: Option<Arc<Scheduler>>, // Exposed scheduler for timers

    // External references and lifecycle management (using placeholders)
    // pub external_referenced_holders: Vec<ExternalReferencedHolderPlaceholder>, // Manages external resources that need lifecycle management.
    // pub trigger_holders: Vec<TriggerPlaceholder>, // Holds trigger runtime instances.
    pub snapshot_service: Option<Arc<SnapshotService>>, // Manages state snapshotting and persistence.
    pub thread_barrier: Option<Arc<ThreadBarrier>>, // Coordinates threads when ordering is enforced.
    pub timestamp_generator: TimestampGeneratorPlaceholder, // Generates timestamps, especially for playback mode. Has a default in Java if not set by user.
    pub id_generator: Option<IdGenerator>, // Generates unique IDs for runtime elements. Option because it's set.

    // pub script_function_map: HashMap<String, ScriptPlaceholder>, // Holds script function implementations.

    // Exception handling (using placeholders)
    // pub disruptor_exception_handler: Option<DisruptorExceptionHandlerPlaceholder>, // Handles exceptions from async event processing.
    // pub runtime_exception_listener: Option<ExceptionListenerPlaceholder>, // Listener for runtime exceptions.
    pub buffer_size: i32, // Default buffer size for event channels.
    // pub included_metrics: Option<Vec<String>>, // Specific metrics to include if statistics are enabled.
    pub transport_channel_creation_enabled: bool, // Whether transport channels (for sources/sinks) should be created.
    // pub scheduler_list: Vec<SchedulerPlaceholder>, // List of schedulers.

    // Simplified placeholder fields for now
    _external_refs_placeholder: Vec<String>,
    _triggers_placeholder: Vec<String>,
    /// Placeholder storage for script functions keyed by name.
    pub script_function_map: HashMap<String, ScriptFunctionPlaceholder>,
    _schedulers_placeholder: Vec<String>,
}

impl SiddhiAppContext {
    // ---- Static flow utilities ----
    pub fn start_group_by_flow(key: String) {
        GROUP_BY_KEY.with(|k| *k.borrow_mut() = Some(key));
    }

    pub fn stop_group_by_flow() {
        GROUP_BY_KEY.with(|k| *k.borrow_mut() = None);
    }

    pub fn start_partition_flow(key: String) {
        PARTITION_KEY.with(|k| *k.borrow_mut() = Some(key));
    }

    pub fn stop_partition_flow() {
        PARTITION_KEY.with(|k| *k.borrow_mut() = None);
    }

    pub fn get_current_flow_id() -> String {
        let partition = PARTITION_KEY.with(|k| k.borrow().clone().unwrap_or_default());
        let group_by = GROUP_BY_KEY.with(|k| k.borrow().clone().unwrap_or_default());
        format!("{}--{}", partition, group_by)
    }

    pub fn get_partition_flow_id() -> Option<String> {
        PARTITION_KEY.with(|k| k.borrow().clone())
    }

    pub fn get_group_by_flow_id() -> Option<String> {
        GROUP_BY_KEY.with(|k| k.borrow().clone())
    }

    // Constructor needs essential parts. Others can be set via builder methods or setters.
    pub fn new(
        siddhi_context: Arc<SiddhiContext>,
        name: String,
        siddhi_app_definition: Arc<SiddhiApp>, // Renamed from siddhi_app to avoid confusion with field
        siddhi_app_string: String,
    ) -> Self {
        // Default values from Java SiddhiAppContext() constructor and field initializers
        let default_buffer = siddhi_context.get_default_junction_buffer_size() as i32;
        Self {
            siddhi_context,
            name,
            siddhi_app_string,
            siddhi_app: siddhi_app_definition,
            is_playback: false,
            is_enforce_order: false, // Default not specified in Java, assuming false
            root_metrics_level: MetricsLevelPlaceholder::default(), // Java defaults to Level.OFF
            statistics_manager: None, // Initialized later in Java
            executor_service: None,
            scheduled_executor_service: None,
            scheduler: None,
            // external_referenced_holders: Vec::new(),
            // trigger_holders: Vec::new(),
            snapshot_service: None,
            thread_barrier: None,
            timestamp_generator: TimestampGeneratorPlaceholder::default(), // Java new-s one if null
            id_generator: None,                                            // Set later
            // script_function_map: HashMap::new(),
            // disruptor_exception_handler: None, // Uses siddhiContext's default if not set
            // runtime_exception_listener: None,
            buffer_size: default_buffer,
            // included_metrics: None,
            transport_channel_creation_enabled: false, // Default not specified, assuming false
            // scheduler_list: Vec::new(),
            _external_refs_placeholder: Vec::new(),
            _triggers_placeholder: Vec::new(),
            script_function_map: HashMap::new(),
            _schedulers_placeholder: Vec::new(),
        }
    }

    /// Register a script function with the given name and return type.
    pub fn add_script_function(
        &mut self,
        name: String,
        return_type: crate::query_api::definition::attribute::Type,
    ) {
        self.script_function_map
            .insert(name, ScriptFunctionPlaceholder { return_type });
    }

    /// Lookup a registered script function placeholder by name.
    pub fn get_script_function(&self, name: &str) -> Option<&ScriptFunctionPlaceholder> {
        self.script_function_map.get(name)
    }

    // --- Getter and Setter examples ---
    pub fn get_siddhi_context(&self) -> Arc<SiddhiContext> {
        Arc::clone(&self.siddhi_context)
    }

    /// Convenience wrapper that lists available scalar function names.
    pub fn list_scalar_function_names(&self) -> Vec<String> {
        self.siddhi_context.list_scalar_function_names()
    }

    /// Convenience wrapper that lists available attribute aggregators.
    pub fn list_attribute_aggregator_names(&self) -> Vec<String> {
        self.siddhi_context.list_attribute_aggregator_names()
    }

    // In Java, getAttributes() delegates to siddhiContext.getAttributes().
    pub fn get_attributes(&self) -> std::sync::RwLockReadGuard<'_, HashMap<String, String>> {
        // Delegate to SiddhiContext's thread-safe attributes map
        self.siddhi_context.get_attributes()
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn is_playback(&self) -> bool {
        self.is_playback
    }

    pub fn set_playback(&mut self, playback: bool) {
        self.is_playback = playback;
    }

    pub fn is_enforce_order(&self) -> bool {
        self.is_enforce_order
    }

    pub fn set_enforce_order(&mut self, enforce: bool) {
        self.is_enforce_order = enforce;
    }

    pub fn get_root_metrics_level(&self) -> MetricsLevelPlaceholder {
        self.root_metrics_level
    }

    pub fn set_root_metrics_level(&mut self, level: MetricsLevelPlaceholder) {
        self.root_metrics_level = level;
    }

    pub fn get_buffer_size(&self) -> i32 {
        self.buffer_size
    }

    pub fn set_buffer_size(&mut self, size: i32) {
        self.buffer_size = size;
    }

    pub fn is_transport_channel_creation_enabled(&self) -> bool {
        self.transport_channel_creation_enabled
    }

    pub fn set_transport_channel_creation_enabled(&mut self, enabled: bool) {
        self.transport_channel_creation_enabled = enabled;
    }

    pub fn get_snapshot_service(&self) -> Option<Arc<SnapshotService>> {
        self.snapshot_service.as_ref().map(Arc::clone)
    }

    pub fn set_snapshot_service(&mut self, service: Arc<SnapshotService>) {
        self.snapshot_service = Some(service);
    }

    pub fn get_thread_barrier(&self) -> Option<Arc<ThreadBarrier>> {
        self.thread_barrier.as_ref().cloned()
    }

    pub fn set_thread_barrier(&mut self, barrier: Arc<ThreadBarrier>) {
        self.thread_barrier = Some(barrier);
    }

    pub fn get_executor_service(&self) -> Option<Arc<ExecutorService>> {
        self.executor_service.as_ref().cloned()
    }

    pub fn set_executor_service(&mut self, service: Arc<ExecutorService>) {
        self.executor_service = Some(service);
    }

    pub fn get_scheduled_executor_service(
        &self,
    ) -> Option<Arc<crate::core::util::ScheduledExecutorService>> {
        self.scheduled_executor_service.as_ref().cloned()
    }

    pub fn set_scheduled_executor_service(
        &mut self,
        service: Arc<crate::core::util::ScheduledExecutorService>,
    ) {
        self.scheduled_executor_service = Some(service);
    }

    pub fn get_scheduler(&self) -> Option<Arc<Scheduler>> {
        self.scheduler.as_ref().cloned()
    }

    pub fn set_scheduler(&mut self, scheduler: Arc<Scheduler>) {
        self.scheduler = Some(scheduler);
    }

    pub fn get_timestamp_generator(&self) -> &TimestampGeneratorPlaceholder {
        &self.timestamp_generator
    }

    pub fn set_timestamp_generator(&mut self, generator: TimestampGeneratorPlaceholder) {
        self.timestamp_generator = generator;
    }

    pub fn set_id_generator(&mut self, id_gen: IdGenerator) {
        self.id_generator = Some(id_gen);
    }
}

#[cfg(test)]
impl SiddhiAppContext {
    /// Convenience constructor used across unit tests to build a minimal
    /// `SiddhiAppContext` instance.  Having this single implementation avoids
    /// multiple `impl` blocks in test modules which previously caused duplicate
    /// method definition errors during compilation.
    pub fn default_for_testing() -> Self {
        use crate::query_api::siddhi_app::SiddhiApp as ApiSiddhiApp;

        Self::new(
            Arc::new(SiddhiContext::default()),
            "test_app_ctx".to_string(),
            Arc::new(ApiSiddhiApp::new("test_api_app".to_string())),
            String::new(),
        )
    }
}
