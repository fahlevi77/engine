// Corresponds to io.siddhi.core.config.SiddhiQueryContext
use super::siddhi_app_context::SiddhiAppContext;
use crate::core::util::id_generator::IdGenerator;
use crate::core::util::state_holder::StateHolder;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::query_api::execution::query::output::OutputEventType;
use std::sync::Arc; // From query_api, as Java uses it.
                    // use crate::core::util::statistics::LatencyTracker; // TODO: Define LatencyTracker
                    // use crate::core::util::IdGenerator; // TODO: Define IdGenerator

// Placeholders
#[derive(Debug, Clone, Default)]
pub struct LatencyTrackerPlaceholder {}

#[derive(Debug)] // Custom Clone below
pub struct SiddhiQueryContext {
    // transient SiddhiAppContext siddhiAppContext in Java
    // Store as Arc<SiddhiAppContext> because SiddhiAppContext is likely shared.
    // Or, if SiddhiQueryContext has a lifetime tied to SiddhiAppContext, it could be a reference.
    // Arc is safer for now.
    pub siddhi_app_context: Arc<SiddhiAppContext>,
    pub name: String,                                       // Query name
    pub partition_id: String,                               // Defaulted in Java if null
    pub partitioned: bool,                                  // Java default false
    pub output_event_type: Option<OutputEventType>,         // Java type, optional
    pub latency_tracker: Option<LatencyTrackerPlaceholder>, // transient in Java, Option in Rust
    pub id_generator: IdGenerator,                          // new-ed in Java constructor
    pub stateful: AtomicBool,                               // whether any state holders registered
}

impl Clone for SiddhiQueryContext {
    fn clone(&self) -> Self {
        Self {
            siddhi_app_context: Arc::clone(&self.siddhi_app_context),
            name: self.name.clone(),
            partition_id: self.partition_id.clone(),
            partitioned: self.partitioned,
            output_event_type: self.output_event_type,
            latency_tracker: self.latency_tracker.clone(),
            id_generator: self.id_generator.clone(),
            stateful: AtomicBool::new(self.stateful.load(Ordering::SeqCst)),
        }
    }
}

impl SiddhiQueryContext {
    pub fn new(
        siddhi_app_context: Arc<SiddhiAppContext>,
        query_name: String,
        partition_id: Option<String>,
    ) -> Self {
        let default_partition_id = "DEFAULT_PARTITION_ID_PLACEHOLDER".to_string(); // TODO: Use SiddhiConstants
        Self {
            siddhi_app_context,
            name: query_name,
            partition_id: partition_id.unwrap_or(default_partition_id),
            partitioned: false,
            output_event_type: None,
            latency_tracker: None,
            id_generator: IdGenerator::default(),
            stateful: AtomicBool::new(false),
        }
    }

    // --- Getters and Setters ---
    pub fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> {
        Arc::clone(&self.siddhi_app_context)
    }

    // In Java, getSiddhiContext() delegates through siddhiAppContext.
    // pub fn get_siddhi_context(&self) -> Arc<SiddhiContext> { // Assuming SiddhiContext is the type
    //     self.siddhi_app_context.get_siddhi_context()
    // }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn is_partitioned(&self) -> bool {
        self.partitioned
    }

    pub fn set_partitioned(&mut self, partitioned: bool) {
        self.partitioned = partitioned;
    }

    pub fn get_output_event_type(&self) -> Option<OutputEventType> {
        self.output_event_type
    }

    pub fn set_output_event_type(&mut self, output_event_type: OutputEventType) {
        self.output_event_type = Some(output_event_type);
    }

    pub fn get_latency_tracker(&self) -> Option<&LatencyTrackerPlaceholder> {
        self.latency_tracker.as_ref()
    }

    pub fn set_latency_tracker(&mut self, tracker: LatencyTrackerPlaceholder) {
        self.latency_tracker = Some(tracker);
    }

    pub fn generate_new_id(&mut self) -> String {
        self.id_generator.create_new_id()
    }

    pub fn is_stateful(&self) -> bool {
        self.stateful.load(Ordering::SeqCst)
    }

    /// Register a state holder with the application's `SnapshotService`.
    /// The provided `name` is namespaced by the query name to ensure uniqueness.
    pub fn register_state_holder(&self, name: String, holder: Arc<dyn StateHolder>) {
        if let Some(service) = self.siddhi_app_context.get_snapshot_service() {
            let key = format!("{}::{}", self.name, name);
            service.register_state_holder(key, holder);
            self.stateful.store(true, Ordering::SeqCst);
        }
    }
}
