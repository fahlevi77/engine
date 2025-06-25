// Corresponds to io.siddhi.core.SiddhiAppRuntime (interface)
// and io.siddhi.core.SiddhiAppRuntimeImpl (implementation)

use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::config::siddhi_query_context::SiddhiQueryContext; // For add_callback
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
use crate::core::util::parser::SiddhiAppParser; // For SiddhiAppParser::parse_siddhi_app_runtime_builder
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
    // TODO: Add other runtime component maps (partitions, triggers)
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
                    "buffer.size" | "buffersize" => {
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
            SiddhiAppParser::parse_siddhi_app_runtime_builder(&api_siddhi_app, siddhi_app_context)?;

        // 3. Build the SiddhiAppRuntime from the builder
        builder.build(api_siddhi_app) // Pass the Arc<ApiSiddhiApp> again
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
            .ok_or_else(|| format!("StreamJunction '{}' not found to add callback", stream_id))?
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
        if let Some(service) = self.siddhi_app_context.get_snapshot_service() {
            if let Some(store) = &service.persistence_store {
                store.clear_all_revisions(&self.name);
            }
        }
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
        service.restore_revision(revision)
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

    // TODO: Implement other methods from SiddhiAppRuntime interface:
    // get_stream_definition_map, get_table_definition_map, etc. (from composed ApiSiddhiApp)
    // get_sources, get_sinks, get_tables (runtime instances), get_queries, get_partitions
    // query(onDemandQueryString), persist, snapshot, restore, etc.
}
