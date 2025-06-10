use siddhi_rust::core::siddhi_manager::SiddhiManager;
use siddhi_rust::core::extension::{WindowProcessorFactory, AttributeAggregatorFactory};
use siddhi_rust::core::query::processor::{CommonProcessorMeta, Processor, ProcessingMode};
use siddhi_rust::core::query::processor::stream::window::WindowProcessor;
use siddhi_rust::core::config::{siddhi_app_context::SiddhiAppContext, siddhi_query_context::SiddhiQueryContext};
use siddhi_rust::core::event::complex_event::ComplexEvent;
use siddhi_rust::query_api::siddhi_app::SiddhiApp;
use siddhi_rust::query_api::execution::query::input::handler::WindowHandler;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::query_api::definition::{StreamDefinition, attribute::Type as AttrType};
use siddhi_rust::core::util::parser::{ExpressionParserContext, parse_expression};
use siddhi_rust::query_api::expression::Expression;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Debug)]
struct DummyWindowProcessor { meta: CommonProcessorMeta }
impl Processor for DummyWindowProcessor {
    fn process(&self, _c: Option<Box<dyn ComplexEvent>>) {}
    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> { None }
    fn set_next_processor(&mut self, _next: Option<Arc<Mutex<dyn Processor>>>) {}
    fn clone_processor(&self, q:&Arc<SiddhiQueryContext>) -> Box<dyn Processor> { Box::new(DummyWindowProcessor{ meta: CommonProcessorMeta::new(Arc::clone(&self.meta.siddhi_app_context), Arc::clone(q)) }) }
    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> { Arc::clone(&self.meta.siddhi_app_context) }
    fn get_processing_mode(&self) -> ProcessingMode { ProcessingMode::BATCH }
    fn is_stateful(&self) -> bool { true }
}
impl WindowProcessor for DummyWindowProcessor {}

#[derive(Debug, Clone)]
struct DummyWindowFactory;
impl WindowProcessorFactory for DummyWindowFactory {
    fn create(&self, _h:&WindowHandler, app:Arc<SiddhiAppContext>, q:Arc<SiddhiQueryContext>) -> Result<Arc<Mutex<dyn Processor>>, String> {
        Ok(Arc::new(Mutex::new(DummyWindowProcessor{ meta: CommonProcessorMeta::new(app,q) })))
    }
    fn clone_box(&self) -> Box<dyn WindowProcessorFactory> { Box::new(Self) }
}

#[derive(Debug, Clone)]
struct ConstAggFactory;
#[derive(Debug, Default)]
struct ConstAggExec;
impl AttributeAggregatorFactory for ConstAggFactory {
    fn create(&self) -> Box<dyn siddhi_rust::core::query::selector::attribute::aggregator::AttributeAggregatorExecutor> { Box::new(ConstAggExec::default()) }
    fn clone_box(&self) -> Box<dyn AttributeAggregatorFactory> { Box::new(Self) }
}
use siddhi_rust::core::query::selector::attribute::aggregator::AttributeAggregatorExecutor;
use siddhi_rust::core::executor::expression_executor::ExpressionExecutor;
impl AttributeAggregatorExecutor for ConstAggExec {
    fn init(&mut self, _e: Vec<Box<dyn ExpressionExecutor>>, _m: ProcessingMode, _ex: bool, _ctx: &SiddhiQueryContext) -> Result<(), String> { Ok(()) }
    fn process_add(&self, _d: Option<AttributeValue>) -> Option<AttributeValue> { Some(AttributeValue::Int(42)) }
    fn process_remove(&self, _d: Option<AttributeValue>) -> Option<AttributeValue> { Some(AttributeValue::Int(42)) }
    fn reset(&self) -> Option<AttributeValue> { Some(AttributeValue::Int(42)) }
    fn clone_box(&self) -> Box<dyn AttributeAggregatorExecutor> { Box::new(ConstAggExec::default()) }
}
impl ExpressionExecutor for ConstAggExec {
    fn execute(&self, _e: Option<&dyn ComplexEvent>) -> Option<AttributeValue> { Some(AttributeValue::Int(42)) }
    fn get_return_type(&self) -> AttrType { AttrType::INT }
    fn clone_executor(&self, _ctx:&Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> { Box::new(ConstAggExec::default()) }
    fn is_attribute_aggregator(&self) -> bool { true }
}

fn make_ctx_with_manager(manager: &SiddhiManager, name:&str) -> ExpressionParserContext<'static> {
    let app = Arc::new(SiddhiApp::new("app".to_string()));
    let app_ctx = Arc::new(SiddhiAppContext::new(manager.siddhi_context(), "app".to_string(), Arc::clone(&app), String::new()));
    let q_ctx = Arc::new(SiddhiQueryContext::new(Arc::clone(&app_ctx), name.to_string(), None));
    let stream_def = Arc::new(StreamDefinition::new("s".to_string()).attribute("v".to_string(), AttrType::INT));
    let meta = siddhi_rust::core::event::stream::meta_stream_event::MetaStreamEvent::new_for_single_input(Arc::clone(&stream_def));
    let mut smap = HashMap::new();
    smap.insert("s".to_string(), Arc::new(meta));
    ExpressionParserContext{ siddhi_app_context: app_ctx, siddhi_query_context: q_ctx, stream_meta_map: smap, table_meta_map: HashMap::new(), default_source: "s".to_string(), query_name: Box::leak(name.to_string().into_boxed_str()) }
}

#[test]
fn test_register_window_factory() {
    let manager = SiddhiManager::new();
    manager.add_window_factory("dummy".to_string(), Box::new(DummyWindowFactory));
    let handler = WindowHandler::new("dummy".to_string(), None, vec![]);
    let app = Arc::new(SiddhiApp::new("A".to_string()));
    let app_ctx = Arc::new(SiddhiAppContext::new(manager.siddhi_context(), "A".to_string(), Arc::clone(&app), String::new()));
    let q_ctx = Arc::new(SiddhiQueryContext::new(Arc::clone(&app_ctx), "q".to_string(), None));
    let res = siddhi_rust::core::query::processor::stream::window::create_window_processor(&handler, app_ctx, q_ctx);
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
