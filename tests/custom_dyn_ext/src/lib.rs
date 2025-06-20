use std::sync::{Arc, Mutex};
use siddhi_rust::core::config::{siddhi_app_context::SiddhiAppContext, siddhi_query_context::SiddhiQueryContext};
use siddhi_rust::core::event::complex_event::ComplexEvent;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::extension::{AttributeAggregatorFactory, WindowProcessorFactory};
use siddhi_rust::core::executor::expression_executor::ExpressionExecutor;
use siddhi_rust::core::query::processor::{CommonProcessorMeta, ProcessingMode, Processor};
use siddhi_rust::core::query::processor::stream::window::WindowProcessor;
use siddhi_rust::core::query::selector::attribute::aggregator::AttributeAggregatorExecutor;
use siddhi_rust::core::siddhi_manager::SiddhiManager;
use siddhi_rust::query_api::definition::attribute::Type as AttrType;
use siddhi_rust::query_api::execution::query::input::handler::WindowHandler;

#[derive(Debug)]
struct DynWindowProcessor {
    meta: CommonProcessorMeta,
}

impl Processor for DynWindowProcessor {
    fn process(&self, chunk: Option<Box<dyn ComplexEvent>>) {
        if let Some(ref next) = self.meta.next_processor {
            next.lock().unwrap().process(chunk);
        }
    }

    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        self.meta.next_processor.as_ref().map(Arc::clone)
    }

    fn set_next_processor(&mut self, next: Option<Arc<Mutex<dyn Processor>>>) {
        self.meta.next_processor = next;
    }

    fn clone_processor(&self, q: &Arc<SiddhiQueryContext>) -> Box<dyn Processor> {
        Box::new(DynWindowProcessor { meta: CommonProcessorMeta::new(Arc::clone(&self.meta.siddhi_app_context), Arc::clone(q)) })
    }

    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> {
        Arc::clone(&self.meta.siddhi_app_context)
    }

    fn get_processing_mode(&self) -> ProcessingMode {
        ProcessingMode::BATCH
    }

    fn is_stateful(&self) -> bool {
        false
    }
}
impl WindowProcessor for DynWindowProcessor {}

#[derive(Debug, Clone)]
struct DynWindowFactory;
impl WindowProcessorFactory for DynWindowFactory {
    fn name(&self) -> &'static str { "dynWindow" }
    fn create(&self, _h: &WindowHandler, app: Arc<SiddhiAppContext>, q: Arc<SiddhiQueryContext>) -> Result<Arc<Mutex<dyn Processor>>, String> {
        Ok(Arc::new(Mutex::new(DynWindowProcessor { meta: CommonProcessorMeta::new(app, q) })))
    }
    fn clone_box(&self) -> Box<dyn WindowProcessorFactory> { Box::new(self.clone()) }
}

#[derive(Debug, Clone)]
struct DynAggFactory;
#[derive(Debug, Default)]
struct DynAggExec;
impl AttributeAggregatorFactory for DynAggFactory {
    fn name(&self) -> &'static str { "dynConstAgg" }
    fn create(&self) -> Box<dyn AttributeAggregatorExecutor> { Box::new(DynAggExec::default()) }
    fn clone_box(&self) -> Box<dyn AttributeAggregatorFactory> { Box::new(self.clone()) }
}
impl AttributeAggregatorExecutor for DynAggExec {
    fn init(&mut self, _e: Vec<Box<dyn ExpressionExecutor>>, _m: ProcessingMode, _ex: bool, _c: &SiddhiQueryContext) -> Result<(), String> { Ok(()) }
    fn process_add(&self, _d: Option<AttributeValue>) -> Option<AttributeValue> { Some(AttributeValue::Int(7)) }
    fn process_remove(&self, _d: Option<AttributeValue>) -> Option<AttributeValue> { Some(AttributeValue::Int(7)) }
    fn reset(&self) -> Option<AttributeValue> { Some(AttributeValue::Int(7)) }
    fn clone_box(&self) -> Box<dyn AttributeAggregatorExecutor> { Box::new(DynAggExec::default()) }
}
impl ExpressionExecutor for DynAggExec {
    fn execute(&self, _e: Option<&dyn ComplexEvent>) -> Option<AttributeValue> { Some(AttributeValue::Int(7)) }
    fn get_return_type(&self) -> AttrType { AttrType::INT }
    fn clone_executor(&self, _c: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> { Box::new(DynAggExec::default()) }
    fn is_attribute_aggregator(&self) -> bool { true }
}

#[no_mangle]
pub extern "C" fn register_extension(manager: &SiddhiManager) {
    manager.add_window_factory("dynWindow".to_string(), Box::new(DynWindowFactory));
    manager.add_attribute_aggregator_factory("dynConstAgg".to_string(), Box::new(DynAggFactory));
}

/// Returns the path to the compiled dynamic library for this crate.
pub fn library_path() -> std::path::PathBuf {
    use std::path::PathBuf;
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("../../target/debug/deps");
    p = p.canonicalize().unwrap_or(p);
    p.push(format!("{}custom_dyn_ext.{}", std::env::consts::DLL_PREFIX, std::env::consts::DLL_EXTENSION));
    p
}
