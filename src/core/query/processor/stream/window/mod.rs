use crate::core::config::{siddhi_app_context::SiddhiAppContext, siddhi_query_context::SiddhiQueryContext};
use crate::core::event::complex_event::ComplexEvent;
use crate::core::query::processor::{CommonProcessorMeta, ProcessingMode, Processor};
use crate::query_api::execution::query::input::handler::WindowHandler;
use crate::core::extension::WindowProcessorFactory;
use crate::query_api::expression::{self, constant::ConstantValueWithFloat, Expression};
use std::sync::{Arc, Mutex};
use crate::core::util::scheduler::{Scheduler, Schedulable};
use crate::core::event::stream::stream_event::StreamEvent;
use crate::core::event::complex_event::ComplexEventType;

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
    scheduler: Option<Arc<Scheduler>>,
}

impl TimeWindowProcessor {
    pub fn new(duration_ms: i64, app_ctx: Arc<SiddhiAppContext>, query_ctx: Arc<SiddhiQueryContext>) -> Self {
        let scheduler = app_ctx.get_scheduler();
        Self { meta: CommonProcessorMeta::new(app_ctx, query_ctx), duration_ms, scheduler }
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

#[derive(Debug)]
struct ExpireTask {
    event: StreamEvent,
    next: Option<Arc<Mutex<dyn Processor>>>,
}

impl Schedulable for ExpireTask {
    fn on_time(&self, timestamp: i64) {
        let mut ev = self.event.clone_without_next();
        ev.set_event_type(ComplexEventType::Expired);
        ev.set_timestamp(timestamp);
        if let Some(ref next) = self.next {
            next.lock().unwrap().process(Some(Box::new(ev)));
        }
    }
}

impl Processor for TimeWindowProcessor {
    fn process(&self, complex_event_chunk: Option<Box<dyn ComplexEvent>>) {
        if let Some(ref next) = self.meta.next_processor {
            if let Some(ref scheduler) = self.scheduler {
                if let Some(ref chunk) = complex_event_chunk {
                    let mut current_opt = Some(chunk.as_ref() as &dyn ComplexEvent);
                    while let Some(ev) = current_opt {
                        if let Some(se) = ev.as_any().downcast_ref::<StreamEvent>() {
                            let task = ExpireTask { event: se.clone_without_next(), next: Some(Arc::clone(next)) };
                            scheduler.notify_at(se.timestamp + self.duration_ms, Arc::new(task));
                        }
                        current_opt = ev.get_next();
                    }
                }
            }
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
    if let Some(factory) = app_ctx
        .get_siddhi_context()
        .get_window_factory(&handler.name)
    {
        factory.create(handler, app_ctx, query_ctx)
    } else {
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
}

#[derive(Debug, Clone)]
pub struct LengthWindowFactory;

impl WindowProcessorFactory for LengthWindowFactory {
    fn create(
        &self,
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Arc<Mutex<dyn Processor>>, String> {
        Ok(Arc::new(Mutex::new(LengthWindowProcessor::from_handler(
            handler,
            app_ctx,
            query_ctx,
        )?)))
    }

    fn clone_box(&self) -> Box<dyn WindowProcessorFactory> {
        Box::new(Self {})
    }
}

#[derive(Debug, Clone)]
pub struct TimeWindowFactory;

impl WindowProcessorFactory for TimeWindowFactory {
    fn create(
        &self,
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Arc<Mutex<dyn Processor>>, String> {
        Ok(Arc::new(Mutex::new(TimeWindowProcessor::from_handler(
            handler,
            app_ctx,
            query_ctx,
        )?)))
    }

    fn clone_box(&self) -> Box<dyn WindowProcessorFactory> {
        Box::new(Self {})
    }
}

