use siddhi_rust::core::event::event::Event;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::persistence::PersistenceStore;
use siddhi_rust::core::siddhi_app_runtime::SiddhiAppRuntime;
use siddhi_rust::core::siddhi_manager::SiddhiManager;
use siddhi_rust::core::stream::input::table_input_handler::TableInputHandler;
use siddhi_rust::core::stream::output::stream_callback::StreamCallback;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct CollectCallback {
    events: Arc<Mutex<Vec<Vec<AttributeValue>>>>,
}

impl StreamCallback for CollectCallback {
    fn receive_events(&self, events: &[Event]) {
        let mut vec = self.events.lock().unwrap();
        for e in events {
            vec.push(e.data.clone());
        }
    }
}

#[derive(Debug)]
pub struct AppRunner {
    runtime: Arc<SiddhiAppRuntime>,
    pub collected: Arc<Mutex<Vec<Vec<AttributeValue>>>>,
    _manager: SiddhiManager,
}

impl AppRunner {
    pub async fn new(app_string: &str, out_stream: &str) -> Self {
        let manager = SiddhiManager::new();
        let runtime = manager
            .create_siddhi_app_runtime_from_string(app_string)
            .await
            .expect("runtime");
        let collected = Arc::new(Mutex::new(Vec::new()));
        runtime
            .add_callback(
                out_stream,
                Box::new(CollectCallback {
                    events: Arc::clone(&collected),
                }),
            )
            .expect("add cb");
        runtime.start();
        Self {
            runtime,
            collected,
            _manager: manager,
        }
    }

    pub async fn new_from_api(
        app: siddhi_rust::query_api::siddhi_app::SiddhiApp,
        out_stream: &str,
    ) -> Self {
        let manager = SiddhiManager::new();
        let runtime = manager
            .create_siddhi_app_runtime_from_api(Arc::new(app), None)
            .await
            .expect("runtime");
        let collected = Arc::new(Mutex::new(Vec::new()));
        runtime
            .add_callback(
                out_stream,
                Box::new(CollectCallback {
                    events: Arc::clone(&collected),
                }),
            )
            .expect("add cb");
        runtime.start();
        Self {
            runtime,
            collected,
            _manager: manager,
        }
    }

    pub async fn new_from_api_with_store(
        app: siddhi_rust::query_api::siddhi_app::SiddhiApp,
        out_stream: &str,
        store: Arc<dyn PersistenceStore>,
    ) -> Self {
        let manager = SiddhiManager::new();
        manager.set_persistence_store(store);
        let runtime = manager
            .create_siddhi_app_runtime_from_api(Arc::new(app), None)
            .await
            .expect("runtime");
        let collected = Arc::new(Mutex::new(Vec::new()));
        runtime
            .add_callback(
                out_stream,
                Box::new(CollectCallback {
                    events: Arc::clone(&collected),
                }),
            )
            .expect("add cb");
        runtime.start();
        Self {
            runtime,
            collected,
            _manager: manager,
        }
    }

    pub async fn new_from_api_with_manager(
        manager: SiddhiManager,
        app: siddhi_rust::query_api::siddhi_app::SiddhiApp,
        out_stream: &str,
    ) -> Self {
        let runtime = manager
            .create_siddhi_app_runtime_from_api(Arc::new(app), None)
            .await
            .expect("runtime");
        let collected = Arc::new(Mutex::new(Vec::new()));
        runtime
            .add_callback(
                out_stream,
                Box::new(CollectCallback {
                    events: Arc::clone(&collected),
                }),
            )
            .expect("add cb");
        runtime.start();
        Self {
            runtime,
            collected,
            _manager: manager,
        }
    }

    pub async fn new_with_manager(manager: SiddhiManager, app_string: &str, out_stream: &str) -> Self {
        let runtime = manager
            .create_siddhi_app_runtime_from_string(app_string)
            .await
            .expect("runtime");
        let collected = Arc::new(Mutex::new(Vec::new()));
        runtime
            .add_callback(
                out_stream,
                Box::new(CollectCallback {
                    events: Arc::clone(&collected),
                }),
            )
            .expect("add cb");
        runtime.start();
        Self {
            runtime,
            collected,
            _manager: manager,
        }
    }

    pub async fn new_with_store(
        app_string: &str,
        out_stream: &str,
        store: Arc<dyn PersistenceStore>,
    ) -> Self {
        let manager = SiddhiManager::new();
        manager.set_persistence_store(store);
        let runtime = manager
            .create_siddhi_app_runtime_from_string(app_string)
            .await
            .expect("runtime");
        let collected = Arc::new(Mutex::new(Vec::new()));
        runtime
            .add_callback(
                out_stream,
                Box::new(CollectCallback {
                    events: Arc::clone(&collected),
                }),
            )
            .expect("add cb");
        runtime.start();
        Self {
            runtime,
            collected,
            _manager: manager,
        }
    }

    pub fn send(&self, stream_id: &str, data: Vec<AttributeValue>) {
        if let Some(handler) = self.runtime.get_input_handler(stream_id) {
            handler
                .lock()
                .unwrap()
                .send_event_with_timestamp(0, data)
                .unwrap();
        }
    }

    /// Send an event with an explicit timestamp.
    pub fn send_with_ts(&self, stream_id: &str, ts: i64, data: Vec<AttributeValue>) {
        if let Some(handler) = self.runtime.get_input_handler(stream_id) {
            handler
                .lock()
                .unwrap()
                .send_event_with_timestamp(ts, data)
                .unwrap();
        }
    }

    pub fn send_batch(&self, stream_id: &str, batch: Vec<Vec<AttributeValue>>) {
        if let Some(handler) = self.runtime.get_input_handler(stream_id) {
            let events: Vec<Event> = batch
                .into_iter()
                .map(|d| Event::new_with_data(0, d))
                .collect();
            handler
                .lock()
                .unwrap()
                .send_multiple_events(events)
                .unwrap();
        }
    }

    pub fn shutdown(self) -> Vec<Vec<AttributeValue>> {
        self.runtime.shutdown();
        self.collected.lock().unwrap().clone()
    }

    pub fn persist(&self) -> String {
        self.runtime.persist().expect("persist")
    }

    pub fn restore_revision(&self, rev: &str) {
        self.runtime.restore_revision(rev).expect("restore")
    }

    pub fn snapshot(&self) -> Vec<u8> {
        self.runtime.snapshot().expect("snapshot")
    }

    pub fn restore(&self, snap: &[u8]) {
        self.runtime.restore(snap).expect("restore")
    }

    pub fn runtime(&self) -> Arc<SiddhiAppRuntime> {
        Arc::clone(&self.runtime)
    }

    /// Obtain an input handler for the given table id if available.
    pub fn get_table_input_handler(&self, table_id: &str) -> Option<TableInputHandler> {
        self.runtime.get_table_input_handler(table_id)
    }

    /// Retrieve aggregated rows using optional `within` and `per` clauses.
    pub fn get_aggregation_data(
        &self,
        agg_id: &str,
        within: Option<siddhi_rust::query_api::aggregation::Within>,
        per: Option<siddhi_rust::query_api::aggregation::time_period::Duration>,
    ) -> Vec<Vec<AttributeValue>> {
        self.runtime.query_aggregation(agg_id, within, per)
    }
}
