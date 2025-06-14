use siddhi_rust::core::config::{
    siddhi_app_context::SiddhiAppContext, siddhi_query_context::SiddhiQueryContext,
};
use siddhi_rust::core::event::complex_event::ComplexEvent;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::extension::{AttributeAggregatorFactory, WindowProcessorFactory};
use siddhi_rust::core::query::processor::stream::window::WindowProcessor;
use siddhi_rust::core::query::processor::{CommonProcessorMeta, ProcessingMode, Processor};
use siddhi_rust::core::siddhi_manager::SiddhiManager;
use siddhi_rust::core::stream::stream_junction::StreamJunction;
use siddhi_rust::core::util::parser::{
    parse_expression, ExpressionParserContext, QueryParser, SiddhiAppParser,
};
use siddhi_rust::query_api::annotation::Annotation;
use siddhi_rust::query_api::definition::{
    attribute::Type as AttrType, StreamDefinition, TableDefinition,
};
use siddhi_rust::query_api::execution::query::input::handler::WindowHandler;
use siddhi_rust::query_api::execution::query::input::stream::input_stream::InputStream;
use siddhi_rust::query_api::execution::query::input::stream::single_input_stream::SingleInputStream;
use siddhi_rust::query_api::execution::query::output::output_stream::{
    InsertIntoStreamAction, OutputStream, OutputStreamAction,
};
use siddhi_rust::query_api::execution::query::selection::Selector;
use siddhi_rust::query_api::execution::query::Query;
use siddhi_rust::query_api::expression::Expression;
use siddhi_rust::query_api::siddhi_app::SiddhiApp;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct DummyWindowProcessor {
    meta: CommonProcessorMeta,
}
impl Processor for DummyWindowProcessor {
    fn process(&self, _c: Option<Box<dyn ComplexEvent>>) {}
    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        None
    }
    fn set_next_processor(&mut self, _next: Option<Arc<Mutex<dyn Processor>>>) {}
    fn clone_processor(&self, q: &Arc<SiddhiQueryContext>) -> Box<dyn Processor> {
        Box::new(DummyWindowProcessor {
            meta: CommonProcessorMeta::new(
                Arc::clone(&self.meta.siddhi_app_context),
                Arc::clone(q),
            ),
        })
    }
    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> {
        Arc::clone(&self.meta.siddhi_app_context)
    }
    fn get_processing_mode(&self) -> ProcessingMode {
        ProcessingMode::BATCH
    }
    fn is_stateful(&self) -> bool {
        true
    }
}
impl WindowProcessor for DummyWindowProcessor {}

#[derive(Debug, Clone)]
struct DummyWindowFactory;
impl WindowProcessorFactory for DummyWindowFactory {
    fn create(
        &self,
        _h: &WindowHandler,
        app: Arc<SiddhiAppContext>,
        q: Arc<SiddhiQueryContext>,
    ) -> Result<Arc<Mutex<dyn Processor>>, String> {
        Ok(Arc::new(Mutex::new(DummyWindowProcessor {
            meta: CommonProcessorMeta::new(app, q),
        })))
    }
    fn clone_box(&self) -> Box<dyn WindowProcessorFactory> {
        Box::new(Self)
    }
}

#[derive(Debug, Clone)]
struct ConstAggFactory;
#[derive(Debug, Default)]
struct ConstAggExec;
impl AttributeAggregatorFactory for ConstAggFactory {
    fn create(
        &self,
    ) -> Box<
        dyn siddhi_rust::core::query::selector::attribute::aggregator::AttributeAggregatorExecutor,
    > {
        Box::new(ConstAggExec::default())
    }
    fn clone_box(&self) -> Box<dyn AttributeAggregatorFactory> {
        Box::new(Self)
    }
}
use siddhi_rust::core::executor::expression_executor::ExpressionExecutor;
use siddhi_rust::core::query::selector::attribute::aggregator::AttributeAggregatorExecutor;
impl AttributeAggregatorExecutor for ConstAggExec {
    fn init(
        &mut self,
        _e: Vec<Box<dyn ExpressionExecutor>>,
        _m: ProcessingMode,
        _ex: bool,
        _ctx: &SiddhiQueryContext,
    ) -> Result<(), String> {
        Ok(())
    }
    fn process_add(&self, _d: Option<AttributeValue>) -> Option<AttributeValue> {
        Some(AttributeValue::Int(42))
    }
    fn process_remove(&self, _d: Option<AttributeValue>) -> Option<AttributeValue> {
        Some(AttributeValue::Int(42))
    }
    fn reset(&self) -> Option<AttributeValue> {
        Some(AttributeValue::Int(42))
    }
    fn clone_box(&self) -> Box<dyn AttributeAggregatorExecutor> {
        Box::new(ConstAggExec::default())
    }
}
impl ExpressionExecutor for ConstAggExec {
    fn execute(&self, _e: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        Some(AttributeValue::Int(42))
    }
    fn get_return_type(&self) -> AttrType {
        AttrType::INT
    }
    fn clone_executor(&self, _ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        Box::new(ConstAggExec::default())
    }
    fn is_attribute_aggregator(&self) -> bool {
        true
    }
}

fn make_ctx_with_manager(manager: &SiddhiManager, name: &str) -> ExpressionParserContext<'static> {
    let app = Arc::new(SiddhiApp::new("app".to_string()));
    let app_ctx = Arc::new(SiddhiAppContext::new(
        manager.siddhi_context(),
        "app".to_string(),
        Arc::clone(&app),
        String::new(),
    ));
    let q_ctx = Arc::new(SiddhiQueryContext::new(
        Arc::clone(&app_ctx),
        name.to_string(),
        None,
    ));
    let stream_def =
        Arc::new(StreamDefinition::new("s".to_string()).attribute("v".to_string(), AttrType::INT));
    let meta =
        siddhi_rust::core::event::stream::meta_stream_event::MetaStreamEvent::new_for_single_input(
            Arc::clone(&stream_def),
        );
    let mut smap = HashMap::new();
    smap.insert("s".to_string(), Arc::new(meta));
    ExpressionParserContext {
        siddhi_app_context: app_ctx,
        siddhi_query_context: q_ctx,
        stream_meta_map: smap,
        table_meta_map: HashMap::new(),
        window_meta_map: HashMap::new(),
        aggregation_meta_map: HashMap::new(),
        state_meta_map: HashMap::new(),
        default_source: "s".to_string(),
        query_name: Box::leak(name.to_string().into_boxed_str()),
    }
}

fn setup_query_env(
    manager: &SiddhiManager,
) -> (
    Arc<SiddhiAppContext>,
    HashMap<String, Arc<Mutex<StreamJunction>>>,
) {
    let ctx = manager.siddhi_context();
    let app = Arc::new(SiddhiApp::new("A".to_string()));
    let app_ctx = Arc::new(SiddhiAppContext::new(
        ctx,
        "A".to_string(),
        Arc::clone(&app),
        String::new(),
    ));

    let in_def = Arc::new(
        StreamDefinition::new("Input".to_string()).attribute("v".to_string(), AttrType::INT),
    );
    let out_def = Arc::new(
        StreamDefinition::new("Out".to_string()).attribute("v".to_string(), AttrType::INT),
    );

    let in_j = Arc::new(Mutex::new(StreamJunction::new(
        "Input".to_string(),
        Arc::clone(&in_def),
        Arc::clone(&app_ctx),
        1024,
        false,
        None,
    )));
    let out_j = Arc::new(Mutex::new(StreamJunction::new(
        "Out".to_string(),
        Arc::clone(&out_def),
        Arc::clone(&app_ctx),
        1024,
        false,
        None,
    )));

    let mut map = HashMap::new();
    map.insert("Input".to_string(), in_j);
    map.insert("Out".to_string(), out_j);
    (app_ctx, map)
}

#[test]
fn test_register_window_factory() {
    let manager = SiddhiManager::new();
    manager.add_window_factory("dummy".to_string(), Box::new(DummyWindowFactory));
    let handler = WindowHandler::new("dummy".to_string(), None, vec![]);
    let app = Arc::new(SiddhiApp::new("A".to_string()));
    let app_ctx = Arc::new(SiddhiAppContext::new(
        manager.siddhi_context(),
        "A".to_string(),
        Arc::clone(&app),
        String::new(),
    ));
    let q_ctx = Arc::new(SiddhiQueryContext::new(
        Arc::clone(&app_ctx),
        "q".to_string(),
        None,
    ));
    let res = siddhi_rust::core::query::processor::stream::window::create_window_processor(
        &handler, app_ctx, q_ctx,
    );
    assert!(res.is_ok());
}

#[test]
fn test_register_attribute_aggregator_factory() {
    let manager = SiddhiManager::new();
    manager.add_attribute_aggregator_factory("constAgg".to_string(), Box::new(ConstAggFactory));
    let ctx = make_ctx_with_manager(&manager, "agg");
    let expr = Expression::function_no_ns("constAgg".to_string(), vec![]);
    let exec = parse_expression(&expr, &ctx).unwrap();
    assert_eq!(exec.execute(None), Some(AttributeValue::Int(42)));
}

#[test]
fn test_query_parser_uses_custom_window_factory() {
    let manager = SiddhiManager::new();
    manager.add_window_factory("dummyWin".to_string(), Box::new(DummyWindowFactory));

    let (app_ctx, junctions) = setup_query_env(&manager);

    let si = SingleInputStream::new_basic("Input".to_string(), false, false, None, Vec::new())
        .window(None, "dummyWin".to_string(), vec![]);
    let input = InputStream::Single(si);
    let selector = Selector::new();
    let insert_action = InsertIntoStreamAction {
        target_id: "Out".to_string(),
        is_inner_stream: false,
        is_fault_stream: false,
    };
    let out_stream = OutputStream::new(OutputStreamAction::InsertInto(insert_action), None);
    let query = Query::query()
        .from(input)
        .select(selector)
        .out_stream(out_stream);

    let runtime = QueryParser::parse_query(
        &query,
        &app_ctx,
        &junctions,
        &HashMap::new(),
        &HashMap::new(),
    )
    .unwrap();
    let head = runtime.processor_chain_head.expect("head");
    let dbg = format!("{:?}", head.lock().unwrap());
    assert!(dbg.contains("DummyWindowProcessor"));
}

#[test]
fn test_table_factory_invoked() {
    static CREATED: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug, Clone)]
    struct RecordingFactory;
    impl siddhi_rust::core::extension::TableFactory for RecordingFactory {
        fn create(
            &self,
            _n: String,
            _p: HashMap<String, String>,
            _c: Arc<siddhi_rust::core::config::siddhi_context::SiddhiContext>,
        ) -> Result<Arc<dyn siddhi_rust::core::table::Table>, String> {
            CREATED.fetch_add(1, Ordering::SeqCst);
            Ok(Arc::new(siddhi_rust::core::table::InMemoryTable::new()))
        }
        fn clone_box(&self) -> Box<dyn siddhi_rust::core::extension::TableFactory> {
            Box::new(self.clone())
        }
    }

    let manager = SiddhiManager::new();
    manager.add_table_factory("rec".to_string(), Box::new(RecordingFactory));

    let ctx = manager.siddhi_context();
    let app = Arc::new(SiddhiApp::new("TApp".to_string()));
    let app_ctx = Arc::new(SiddhiAppContext::new(
        Arc::clone(&ctx),
        "TApp".to_string(),
        Arc::clone(&app),
        String::new(),
    ));

    let table_def = TableDefinition::id("T1".to_string())
        .attribute("v".to_string(), AttrType::INT)
        .annotation(
            Annotation::new("store".to_string())
                .element(Some("type".to_string()), "rec".to_string()),
        );

    let mut app_obj = SiddhiApp::new("TApp".to_string());
    app_obj
        .table_definition_map
        .insert("T1".to_string(), Arc::new(table_def));

    let _ =
        SiddhiAppParser::parse_siddhi_app_runtime_builder(&app_obj, Arc::clone(&app_ctx)).unwrap();

    assert_eq!(CREATED.load(Ordering::SeqCst), 1);
    assert!(app_ctx.get_siddhi_context().get_table("T1").is_some());
}
