use siddhi_rust::core::config::{
    siddhi_app_context::{MetricsLevelPlaceholder, SiddhiAppContext},
    siddhi_context::SiddhiContext,
};
use siddhi_rust::core::event::event::Event;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::stream::output::InMemoryErrorStore;
use siddhi_rust::core::query::processor::Processor;
use siddhi_rust::core::stream::stream_junction::{OnErrorAction, StreamJunction};
use siddhi_rust::core::config::siddhi_query_context::SiddhiQueryContext;
use siddhi_rust::query_api::definition::{attribute::Type as AttrType, StreamDefinition};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct RecordingProcessor {
    events: Arc<Mutex<Vec<Vec<AttributeValue>>>>,
}

impl Processor for RecordingProcessor {
    fn process(
        &self,
        mut chunk: Option<Box<dyn siddhi_rust::core::event::complex_event::ComplexEvent>>,
    ) {
        while let Some(mut ce) = chunk {
            chunk = ce.set_next(None);
            if let Some(se) = ce
                .as_any()
                .downcast_ref::<siddhi_rust::core::event::stream::StreamEvent>()
            {
                self.events
                    .lock()
                    .unwrap()
                    .push(se.before_window_data.clone());
            }
        }
    }
    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        None
    }
    fn set_next_processor(&mut self, _n: Option<Arc<Mutex<dyn Processor>>>) {}
    fn clone_processor(
        &self,
        _c: &Arc<siddhi_rust::core::config::siddhi_query_context::SiddhiQueryContext>,
    ) -> Box<dyn Processor> {
        Box::new(RecordingProcessor {
            events: Arc::clone(&self.events),
        })
    }
    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> {
        Arc::new(SiddhiAppContext::new(
            Arc::new(SiddhiContext::new()),
            "T".to_string(),
            Arc::new(siddhi_rust::query_api::siddhi_app::SiddhiApp::new(
                "T".to_string(),
            )),
            String::new(),
        ))
    }

    fn get_siddhi_query_context(&self) -> Arc<SiddhiQueryContext> {
        Arc::new(SiddhiQueryContext::new(
            Arc::new(SiddhiAppContext::new(
                Arc::new(SiddhiContext::new()),
                "T".to_string(),
                Arc::new(siddhi_rust::query_api::siddhi_app::SiddhiApp::new(
                    "T".to_string(),
                )),
                String::new(),
            )),
            "q".to_string(),
            None,
        ))
    }
    fn get_processing_mode(&self) -> siddhi_rust::core::query::processor::ProcessingMode {
        siddhi_rust::core::query::processor::ProcessingMode::DEFAULT
    }
    fn is_stateful(&self) -> bool {
        false
    }
}

#[test]
fn test_fault_stream_routing() {
    let siddhi_context = Arc::new(SiddhiContext::new());
    let app = Arc::new(siddhi_rust::query_api::siddhi_app::SiddhiApp::new(
        "App".to_string(),
    ));
    let mut app_ctx = SiddhiAppContext::new(
        Arc::clone(&siddhi_context),
        "App".to_string(),
        Arc::clone(&app),
        String::new(),
    );
    app_ctx.set_root_metrics_level(MetricsLevelPlaceholder::BASIC);
    let def =
        Arc::new(StreamDefinition::new("In".to_string()).attribute("a".to_string(), AttrType::INT));
    let fault_def = Arc::new(
        StreamDefinition::new("Fault".to_string()).attribute("a".to_string(), AttrType::INT),
    );
    let fault_junction = Arc::new(Mutex::new(StreamJunction::new(
        "Fault".to_string(),
        Arc::clone(&fault_def),
        Arc::new(app_ctx.clone()),
        8,
        false,
        None,
    )));
    let mut main_junction = StreamJunction::new(
        "In".to_string(),
        Arc::clone(&def),
        Arc::new(app_ctx.clone()),
        8,
        true,
        Some(Arc::clone(&fault_junction)),
    );
    main_junction.set_on_error_action(OnErrorAction::STREAM);
    let rec = Arc::new(Mutex::new(Vec::new()));
    fault_junction
        .lock()
        .unwrap()
        .subscribe(Arc::new(Mutex::new(RecordingProcessor {
            events: Arc::clone(&rec),
        })));
    main_junction.stop_processing();
    main_junction.send_event(Event::new_with_data(0, vec![AttributeValue::Int(1)]));
    assert_eq!(rec.lock().unwrap().len(), 1);
}

#[test]
fn test_error_store_routing() {
    let mut siddhi_context = SiddhiContext::new();
    let error_store = Arc::new(InMemoryErrorStore::new());
    siddhi_context.set_error_store(error_store.clone());
    let siddhi_context = Arc::new(siddhi_context);
    let app = Arc::new(siddhi_rust::query_api::siddhi_app::SiddhiApp::new(
        "App".to_string(),
    ));
    let app_ctx = SiddhiAppContext::new(
        Arc::clone(&siddhi_context),
        "App".to_string(),
        Arc::clone(&app),
        String::new(),
    );
    let def =
        Arc::new(StreamDefinition::new("In".to_string()).attribute("a".to_string(), AttrType::INT));
    let mut junction = StreamJunction::new(
        "In".to_string(),
        Arc::clone(&def),
        Arc::new(app_ctx),
        8,
        true,
        None,
    );
    junction.set_on_error_action(OnErrorAction::STORE);
    junction.stop_processing();
    junction.send_event(Event::new_with_data(0, vec![AttributeValue::Int(2)]));
    assert_eq!(error_store.errors().len(), 1);
}
