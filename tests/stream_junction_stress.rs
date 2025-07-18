use siddhi_rust::core::config::{
    siddhi_app_context::SiddhiAppContext, siddhi_context::SiddhiContext,
};
use siddhi_rust::core::event::complex_event::ComplexEvent;
use siddhi_rust::core::event::event::Event;
use siddhi_rust::core::event::stream::StreamEvent;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::query::processor::Processor;
use siddhi_rust::core::stream::stream_junction::StreamJunction;
use siddhi_rust::core::config::siddhi_query_context::SiddhiQueryContext;
use siddhi_rust::query_api::definition::attribute::Type as AttrType;
use siddhi_rust::query_api::definition::StreamDefinition;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct RecordingProcessor {
    events: Arc<Mutex<Vec<Vec<AttributeValue>>>>,
}

impl Processor for RecordingProcessor {
    fn process(&self, mut chunk: Option<Box<dyn ComplexEvent>>) {
        while let Some(mut ce) = chunk {
            chunk = ce.set_next(None);
            if let Some(se) = ce.as_any().downcast_ref::<StreamEvent>() {
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

fn setup_junction(
    async_mode: bool,
) -> (
    Arc<Mutex<StreamJunction>>,
    Arc<Mutex<Vec<Vec<AttributeValue>>>>,
) {
    let siddhi_context = Arc::new(SiddhiContext::new());
    let app = Arc::new(siddhi_rust::query_api::siddhi_app::SiddhiApp::new(
        "App".to_string(),
    ));
    let mut ctx = SiddhiAppContext::new(
        Arc::clone(&siddhi_context),
        "App".to_string(),
        Arc::clone(&app),
        String::new(),
    );
    ctx.root_metrics_level =
        siddhi_rust::core::config::siddhi_app_context::MetricsLevelPlaceholder::BASIC;
    let app_ctx = Arc::new(ctx);
    let def = Arc::new(
        StreamDefinition::new("InputStream".to_string()).attribute("a".to_string(), AttrType::INT),
    );
    let junction = Arc::new(Mutex::new(StreamJunction::new(
        "InputStream".to_string(),
        Arc::clone(&def),
        Arc::clone(&app_ctx),
        1024,
        async_mode,
        None,
    )));
    let rec = Arc::new(Mutex::new(Vec::new()));
    let p = Arc::new(Mutex::new(RecordingProcessor {
        events: Arc::clone(&rec),
    }));
    junction.lock().unwrap().subscribe(p);
    (junction, rec)
}

#[test]
fn stress_async_concurrent_publish() {
    let (junction, rec) = setup_junction(true);
    let mut handles = Vec::new();
    for i in 0..4 {
        let j = junction.clone();
        handles.push(thread::spawn(move || {
            for k in 0..500 {
                j.lock().unwrap().send_event(Event::new_with_data(
                    0,
                    vec![AttributeValue::Int(i * 500 + k)],
                ));
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    thread::sleep(Duration::from_millis(500));
    let data = rec.lock().unwrap();
    assert_eq!(data.len(), 2000);
    // metrics
    let count = junction.lock().unwrap().total_events().unwrap_or(0);
    assert_eq!(count, 2000);
}
