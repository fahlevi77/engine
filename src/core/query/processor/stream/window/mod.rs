use crate::core::config::{siddhi_app_context::SiddhiAppContext, siddhi_query_context::SiddhiQueryContext};
use crate::core::event::complex_event::ComplexEvent;
use crate::core::query::processor::{CommonProcessorMeta, ProcessingMode, Processor};
use crate::query_api::execution::query::input::handler::WindowHandler;
use crate::query_api::expression::{self, constant::ConstantValueWithFloat, Expression};
use std::sync::{Arc, Mutex};

pub trait WindowProcessor: Processor {}

#[derive(Debug)]
pub struct LengthWindowProcessor {
    meta: CommonProcessorMeta,
    pub length: usize,
}

impl LengthWindowProcessor {
    pub fn new(length: usize, app_ctx: Arc<SiddhiAppContext>, query_ctx: Arc<SiddhiQueryContext>) -> Self {
        Self { meta: CommonProcessorMeta::new(app_ctx, query_ctx), length }
    }

    pub fn from_handler(
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Self, String> {
        let expr = handler
            .get_parameters()
            .first()
            .ok_or("Length window requires a parameter")?;
        if let Expression::Constant(c) = expr {
            let len = match &c.value {
                ConstantValueWithFloat::Int(i) => *i as usize,
                ConstantValueWithFloat::Long(l) => *l as usize,
                _ => {
                    return Err("Length window size must be int or long".to_string())
                }
            };
            Ok(Self::new(len, app_ctx, query_ctx))
        } else {
            Err("Length window size must be constant".to_string())
        }
    }
}

impl Processor for LengthWindowProcessor {
    fn process(&self, complex_event_chunk: Option<Box<dyn ComplexEvent>>) {
        if let Some(ref next) = self.meta.next_processor {
            next.lock().unwrap().process(complex_event_chunk);
        }
    }

    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        self.meta.next_processor.as_ref().map(Arc::clone)
    }

    fn set_next_processor(&mut self, next: Option<Arc<Mutex<dyn Processor>>>) {
        self.meta.next_processor = next;
    }

    fn clone_processor(&self, query_ctx: &Arc<SiddhiQueryContext>) -> Box<dyn Processor> {
        Box::new(Self::new(self.length, Arc::clone(&self.meta.siddhi_app_context), Arc::clone(query_ctx)))
    }

    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> {
        Arc::clone(&self.meta.siddhi_app_context)
    }

    fn get_processing_mode(&self) -> ProcessingMode { ProcessingMode::SLIDE }

    fn is_stateful(&self) -> bool { true }
}

impl WindowProcessor for LengthWindowProcessor {}

#[derive(Debug)]
pub struct TimeWindowProcessor {
    meta: CommonProcessorMeta,
    pub duration_ms: i64,
}

impl TimeWindowProcessor {
    pub fn new(duration_ms: i64, app_ctx: Arc<SiddhiAppContext>, query_ctx: Arc<SiddhiQueryContext>) -> Self {
        Self { meta: CommonProcessorMeta::new(app_ctx, query_ctx), duration_ms }
    }

    pub fn from_handler(
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Self, String> {
        let expr = handler
            .get_parameters()
            .first()
            .ok_or("Time window requires a parameter")?;
        if let Expression::Constant(c) = expr {
            let dur = match &c.value {
                ConstantValueWithFloat::Time(t) => *t,
                ConstantValueWithFloat::Long(l) => *l,
                _ => return Err("Time window duration must be time constant".to_string()),
            };
            Ok(Self::new(dur, app_ctx, query_ctx))
        } else {
            Err("Time window duration must be constant".to_string())
        }
    }
}

impl Processor for TimeWindowProcessor {
    fn process(&self, complex_event_chunk: Option<Box<dyn ComplexEvent>>) {
        if let Some(ref next) = self.meta.next_processor {
            next.lock().unwrap().process(complex_event_chunk);
        }
    }

    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        self.meta.next_processor.as_ref().map(Arc::clone)
    }

    fn set_next_processor(&mut self, next: Option<Arc<Mutex<dyn Processor>>>) {
        self.meta.next_processor = next;
    }

    fn clone_processor(&self, query_ctx: &Arc<SiddhiQueryContext>) -> Box<dyn Processor> {
        Box::new(Self::new(self.duration_ms, Arc::clone(&self.meta.siddhi_app_context), Arc::clone(query_ctx)))
    }

    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> {
        Arc::clone(&self.meta.siddhi_app_context)
    }

    fn get_processing_mode(&self) -> ProcessingMode { ProcessingMode::SLIDE }

    fn is_stateful(&self) -> bool { true }
}

impl WindowProcessor for TimeWindowProcessor {}

pub fn create_window_processor(
    handler: &WindowHandler,
    app_ctx: Arc<SiddhiAppContext>,
    query_ctx: Arc<SiddhiQueryContext>,
) -> Result<Arc<Mutex<dyn Processor>>, String> {
    match handler.name.as_str() {
        "length" => Ok(Arc::new(Mutex::new(LengthWindowProcessor::from_handler(
            handler,
            app_ctx,
            query_ctx,
        )?))),
        "time" => Ok(Arc::new(Mutex::new(TimeWindowProcessor::from_handler(
            handler,
            app_ctx,
            query_ctx,
        )?))),
        other => Err(format!("Unsupported window type '{}'", other)),
    }
}

