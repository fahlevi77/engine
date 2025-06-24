use crate::core::config::{
    siddhi_app_context::SiddhiAppContext, siddhi_query_context::SiddhiQueryContext,
};
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::complex_event::ComplexEventType;
use crate::core::event::stream::stream_event::StreamEvent;
use crate::core::extension::WindowProcessorFactory;
use crate::core::query::processor::{CommonProcessorMeta, ProcessingMode, Processor};
use crate::core::util::scheduler::{Schedulable, Scheduler};
use crate::core::util::state_holder::StateHolder;
use crate::core::util::{event_from_bytes, event_to_bytes, from_bytes, to_bytes};
use crate::query_api::execution::query::input::handler::WindowHandler;
use crate::query_api::expression::{self, constant::ConstantValueWithFloat, Expression};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub trait WindowProcessor: Processor {}

#[derive(Debug)]
pub struct LengthWindowProcessor {
    meta: CommonProcessorMeta,
    pub length: usize,
    buffer: Arc<Mutex<VecDeque<Arc<StreamEvent>>>>,
}

#[derive(Debug)]
struct LengthWindowStateHolder {
    buffer: Arc<Mutex<VecDeque<Arc<StreamEvent>>>>,
}

impl StateHolder for LengthWindowStateHolder {
    fn snapshot_state(&self) -> Vec<u8> {
        let buf = self.buffer.lock().unwrap();
        let events: Vec<Vec<u8>> = buf
            .iter()
            .map(|e| {
                let mut ev = crate::core::event::event::Event::new_with_data(
                    e.timestamp,
                    e.before_window_data.clone(),
                );
                ev.is_expired = e.event_type == ComplexEventType::Expired;
                event_to_bytes(&ev).unwrap_or_default()
            })
            .collect();
        to_bytes(&events).unwrap_or_default()
    }

    fn restore_state(&self, snapshot: &[u8]) {
        if let Ok(ev_bytes) = from_bytes::<Vec<Vec<u8>>>(snapshot) {
            let mut buf = self.buffer.lock().unwrap();
            buf.clear();
            for b in ev_bytes {
                if let Ok(ev) = event_from_bytes(&b) {
                    let mut se = StreamEvent::new(ev.timestamp, ev.data.len(), 0, 0);
                    se.before_window_data = ev.data;
                    se.event_type = if ev.is_expired {
                        ComplexEventType::Expired
                    } else {
                        ComplexEventType::Current
                    };
                    buf.push_back(Arc::new(se));
                }
            }
        }
    }
}

impl LengthWindowProcessor {
    pub fn new(
        length: usize,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Self {
        let buffer = Arc::new(Mutex::new(VecDeque::new()));
        let holder = Arc::new(LengthWindowStateHolder {
            buffer: Arc::clone(&buffer),
        });
        query_ctx.register_state_holder("length_window".to_string(), holder);
        Self {
            meta: CommonProcessorMeta::new(app_ctx, query_ctx),
            length,
            buffer,
        }
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
                _ => return Err("Length window size must be int or long".to_string()),
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
            if let Some(ref chunk) = complex_event_chunk {
                let mut current_opt = Some(chunk.as_ref() as &dyn ComplexEvent);
                while let Some(ev) = current_opt {
                    if let Some(se) = ev.as_any().downcast_ref::<StreamEvent>() {
                        let mut expired: Option<Box<dyn ComplexEvent>> = None;
                        {
                            let mut buf = self.buffer.lock().unwrap();
                            if buf.len() >= self.length {
                                if let Some(old) = buf.pop_front() {
                                    let mut ex = old.as_ref().clone_without_next();
                                    ex.set_event_type(ComplexEventType::Expired);
                                    ex.set_timestamp(se.timestamp);
                                    expired = Some(Box::new(ex));
                                }
                            }
                            buf.push_back(Arc::new(se.clone_without_next()));
                        }
                        if let Some(mut ex) = expired {
                            let tail = ex.mut_next_ref_option();
                            *tail = Some(Box::new(se.clone_without_next()));
                            next.lock().unwrap().process(Some(ex));
                        } else {
                            next.lock()
                                .unwrap()
                                .process(Some(Box::new(se.clone_without_next())));
                        }
                    }
                    current_opt = ev.get_next();
                }
            } else {
                next.lock().unwrap().process(None);
            }
        }
    }

    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        self.meta.next_processor.as_ref().map(Arc::clone)
    }

    fn set_next_processor(&mut self, next: Option<Arc<Mutex<dyn Processor>>>) {
        self.meta.next_processor = next;
    }

    fn clone_processor(&self, query_ctx: &Arc<SiddhiQueryContext>) -> Box<dyn Processor> {
        Box::new(Self::new(
            self.length,
            Arc::clone(&self.meta.siddhi_app_context),
            Arc::clone(query_ctx),
        ))
    }

    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> {
        Arc::clone(&self.meta.siddhi_app_context)
    }

    fn get_processing_mode(&self) -> ProcessingMode {
        ProcessingMode::SLIDE
    }

    fn is_stateful(&self) -> bool {
        true
    }
}

impl WindowProcessor for LengthWindowProcessor {}

#[derive(Debug)]
pub struct TimeWindowProcessor {
    meta: CommonProcessorMeta,
    pub duration_ms: i64,
    scheduler: Option<Arc<Scheduler>>,
    buffer: Arc<Mutex<VecDeque<Arc<StreamEvent>>>>,
}

impl TimeWindowProcessor {
    pub fn new(
        duration_ms: i64,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Self {
        let scheduler = app_ctx.get_scheduler();
        Self {
            meta: CommonProcessorMeta::new(app_ctx, query_ctx),
            duration_ms,
            scheduler,
            buffer: Arc::new(Mutex::new(VecDeque::new())),
        }
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
    event: Arc<StreamEvent>,
    buffer: Arc<Mutex<VecDeque<Arc<StreamEvent>>>>,
    next: Option<Arc<Mutex<dyn Processor>>>,
}

impl Schedulable for ExpireTask {
    fn on_time(&self, timestamp: i64) {
        let ev_arc = {
            let mut buf = self.buffer.lock().unwrap();
            if let Some(pos) = buf.iter().position(|e| Arc::ptr_eq(e, &self.event)) {
                buf.remove(pos)
            } else {
                None
            }
        };
        if let Some(ev_arc) = ev_arc {
            let mut ev = ev_arc.as_ref().clone_without_next();
            ev.set_event_type(ComplexEventType::Expired);
            ev.set_timestamp(timestamp);
            if let Some(ref next) = self.next {
                next.lock().unwrap().process(Some(Box::new(ev)));
            }
        }
    }
}

#[derive(Clone)]
struct BatchFlushTask {
    buffer: Arc<Mutex<Vec<StreamEvent>>>,
    expired: Arc<Mutex<Vec<StreamEvent>>>,
    next: Option<Arc<Mutex<dyn Processor>>>,
    duration_ms: i64,
    scheduler: Arc<Scheduler>,
    start_time: Arc<Mutex<Option<i64>>>,
}

impl Schedulable for BatchFlushTask {
    fn on_time(&self, timestamp: i64) {
        let expired_batch: Vec<StreamEvent> = {
            let mut ex = self.expired.lock().unwrap();
            std::mem::take(&mut *ex)
        };
        let current_batch: Vec<StreamEvent> = {
            let mut buf = self.buffer.lock().unwrap();
            std::mem::take(&mut *buf)
        };

        if expired_batch.is_empty() && current_batch.is_empty() {
            *self.start_time.lock().unwrap() = Some(timestamp);
            self.scheduler
                .notify_at(timestamp + self.duration_ms, Arc::new(self.clone()));
            return;
        }

        let mut head: Option<Box<dyn ComplexEvent>> = None;
        let mut tail = &mut head;

        for mut e in expired_batch {
            e.set_event_type(ComplexEventType::Expired);
            e.set_timestamp(timestamp);
            *tail = Some(Box::new(e.clone_without_next()));
            tail = tail.as_mut().unwrap().mut_next_ref_option();
        }

        for e in &current_batch {
            *tail = Some(Box::new(e.clone_without_next()));
            tail = tail.as_mut().unwrap().mut_next_ref_option();
        }

        {
            let mut ex = self.expired.lock().unwrap();
            ex.extend(current_batch);
        }

        if let Some(chain) = head {
            if let Some(ref next) = self.next {
                next.lock().unwrap().process(Some(chain));
            }
        }

        *self.start_time.lock().unwrap() = Some(timestamp);
        self.scheduler
            .notify_at(timestamp + self.duration_ms, Arc::new(self.clone()));
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
                            let arc = Arc::new(se.clone_without_next());
                            {
                                let mut buf = self.buffer.lock().unwrap();
                                buf.push_back(Arc::clone(&arc));
                            }
                            let task = ExpireTask {
                                event: Arc::clone(&arc),
                                buffer: Arc::clone(&self.buffer),
                                next: Some(Arc::clone(next)),
                            };
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
        Box::new(Self::new(
            self.duration_ms,
            Arc::clone(&self.meta.siddhi_app_context),
            Arc::clone(query_ctx),
        ))
    }

    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> {
        Arc::clone(&self.meta.siddhi_app_context)
    }

    fn get_processing_mode(&self) -> ProcessingMode {
        ProcessingMode::SLIDE
    }

    fn is_stateful(&self) -> bool {
        true
    }
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
                handler, app_ctx, query_ctx,
            )?))),
            "time" => Ok(Arc::new(Mutex::new(TimeWindowProcessor::from_handler(
                handler, app_ctx, query_ctx,
            )?))),
            "lengthBatch" => Ok(Arc::new(Mutex::new(
                LengthBatchWindowProcessor::from_handler(handler, app_ctx, query_ctx)?,
            ))),
            "timeBatch" => Ok(Arc::new(Mutex::new(
                TimeBatchWindowProcessor::from_handler(handler, app_ctx, query_ctx)?,
            ))),
            "externalTime" => Ok(Arc::new(Mutex::new(
                ExternalTimeWindowProcessor::from_handler(handler, app_ctx, query_ctx)?,
            ))),
            "externalTimeBatch" => Ok(Arc::new(Mutex::new(
                ExternalTimeBatchWindowProcessor::from_handler(handler, app_ctx, query_ctx)?,
            ))),
            other => Err(format!("Unsupported window type '{}'", other)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LengthWindowFactory;

impl WindowProcessorFactory for LengthWindowFactory {
    fn name(&self) -> &'static str {
        "length"
    }
    fn create(
        &self,
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Arc<Mutex<dyn Processor>>, String> {
        Ok(Arc::new(Mutex::new(LengthWindowProcessor::from_handler(
            handler, app_ctx, query_ctx,
        )?)))
    }

    fn clone_box(&self) -> Box<dyn WindowProcessorFactory> {
        Box::new(Self {})
    }
}

#[derive(Debug, Clone)]
pub struct TimeWindowFactory;

impl WindowProcessorFactory for TimeWindowFactory {
    fn name(&self) -> &'static str {
        "time"
    }
    fn create(
        &self,
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Arc<Mutex<dyn Processor>>, String> {
        Ok(Arc::new(Mutex::new(TimeWindowProcessor::from_handler(
            handler, app_ctx, query_ctx,
        )?)))
    }

    fn clone_box(&self) -> Box<dyn WindowProcessorFactory> {
        Box::new(Self {})
    }
}

// ---- LengthBatchWindowProcessor ----

#[derive(Debug)]
pub struct LengthBatchWindowProcessor {
    meta: CommonProcessorMeta,
    pub length: usize,
    buffer: Arc<Mutex<Vec<StreamEvent>>>,
    expired: Arc<Mutex<Vec<StreamEvent>>>,
}

impl LengthBatchWindowProcessor {
    pub fn new(
        length: usize,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Self {
        Self {
            meta: CommonProcessorMeta::new(app_ctx, query_ctx),
            length,
            buffer: Arc::new(Mutex::new(Vec::new())),
            expired: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn from_handler(
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Self, String> {
        let expr = handler
            .get_parameters()
            .first()
            .ok_or("LengthBatch window requires a parameter")?;
        if let Expression::Constant(c) = expr {
            let len = match &c.value {
                ConstantValueWithFloat::Int(i) => *i as usize,
                ConstantValueWithFloat::Long(l) => *l as usize,
                _ => return Err("LengthBatch window size must be int or long".to_string()),
            };
            Ok(Self::new(len, app_ctx, query_ctx))
        } else {
            Err("LengthBatch window size must be constant".to_string())
        }
    }

    fn flush(&self, timestamp: i64) {
        if let Some(ref next) = self.meta.next_processor {
            let expired_batch: Vec<StreamEvent> = {
                let mut ex = self.expired.lock().unwrap();
                std::mem::take(&mut *ex)
            };
            let current_batch: Vec<StreamEvent> = {
                let mut buf = self.buffer.lock().unwrap();
                std::mem::take(&mut *buf)
            };

            if expired_batch.is_empty() && current_batch.is_empty() {
                return;
            }

            let mut head: Option<Box<dyn ComplexEvent>> = None;
            let mut tail = &mut head;

            for mut e in expired_batch {
                e.set_event_type(ComplexEventType::Expired);
                e.set_timestamp(timestamp);
                *tail = Some(Box::new(e.clone_without_next()));
                tail = tail.as_mut().unwrap().mut_next_ref_option();
            }

            for e in &current_batch {
                *tail = Some(Box::new(e.clone_without_next()));
                tail = tail.as_mut().unwrap().mut_next_ref_option();
            }

            {
                let mut ex = self.expired.lock().unwrap();
                ex.extend(current_batch);
            }

            if let Some(chain) = head {
                next.lock().unwrap().process(Some(chain));
            }
        }
    }
}

impl Processor for LengthBatchWindowProcessor {
    fn process(&self, complex_event_chunk: Option<Box<dyn ComplexEvent>>) {
        if let Some(chunk) = complex_event_chunk {
            let mut current_opt = Some(chunk.as_ref() as &dyn ComplexEvent);
            let mut last_ts = 0i64;
            while let Some(ev) = current_opt {
                if let Some(se) = ev.as_any().downcast_ref::<StreamEvent>() {
                    self.buffer.lock().unwrap().push(se.clone_without_next());
                    last_ts = se.timestamp;
                    if self.buffer.lock().unwrap().len() >= self.length {
                        self.flush(last_ts);
                    }
                }
                current_opt = ev.get_next();
            }
        }
    }

    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        self.meta.next_processor.as_ref().map(Arc::clone)
    }

    fn set_next_processor(&mut self, next: Option<Arc<Mutex<dyn Processor>>>) {
        self.meta.next_processor = next;
    }

    fn clone_processor(&self, ctx: &Arc<SiddhiQueryContext>) -> Box<dyn Processor> {
        Box::new(Self::new(
            self.length,
            Arc::clone(&self.meta.siddhi_app_context),
            Arc::clone(ctx),
        ))
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

impl WindowProcessor for LengthBatchWindowProcessor {}

#[derive(Debug, Clone)]
pub struct LengthBatchWindowFactory;

impl WindowProcessorFactory for LengthBatchWindowFactory {
    fn name(&self) -> &'static str {
        "lengthBatch"
    }
    fn create(
        &self,
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Arc<Mutex<dyn Processor>>, String> {
        Ok(Arc::new(Mutex::new(
            LengthBatchWindowProcessor::from_handler(handler, app_ctx, query_ctx)?,
        )))
    }

    fn clone_box(&self) -> Box<dyn WindowProcessorFactory> {
        Box::new(Self {})
    }
}

// ---- TimeBatchWindowProcessor ----

#[derive(Debug)]
pub struct TimeBatchWindowProcessor {
    meta: CommonProcessorMeta,
    pub duration_ms: i64,
    scheduler: Option<Arc<Scheduler>>,
    buffer: Arc<Mutex<Vec<StreamEvent>>>,
    expired: Arc<Mutex<Vec<StreamEvent>>>,
    start_time: Arc<Mutex<Option<i64>>>,
}

impl TimeBatchWindowProcessor {
    pub fn new(
        duration_ms: i64,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Self {
        let scheduler = app_ctx.get_scheduler();
        Self {
            meta: CommonProcessorMeta::new(app_ctx, query_ctx),
            duration_ms,
            scheduler,
            buffer: Arc::new(Mutex::new(Vec::new())),
            expired: Arc::new(Mutex::new(Vec::new())),
            start_time: Arc::new(Mutex::new(None)),
        }
    }

    pub fn from_handler(
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Self, String> {
        let expr = handler
            .get_parameters()
            .first()
            .ok_or("TimeBatch window requires a parameter")?;
        if let Expression::Constant(c) = expr {
            let dur = match &c.value {
                ConstantValueWithFloat::Time(t) => *t,
                ConstantValueWithFloat::Long(l) => *l,
                _ => return Err("TimeBatch window duration must be time constant".to_string()),
            };
            Ok(Self::new(dur, app_ctx, query_ctx))
        } else {
            Err("TimeBatch window duration must be constant".to_string())
        }
    }

    fn flush(&self, timestamp: i64) {
        if let Some(ref next) = self.meta.next_processor {
            let expired_batch: Vec<StreamEvent> = {
                let mut ex = self.expired.lock().unwrap();
                std::mem::take(&mut *ex)
            };
            let current_batch: Vec<StreamEvent> = {
                let mut buf = self.buffer.lock().unwrap();
                std::mem::take(&mut *buf)
            };

            if expired_batch.is_empty() && current_batch.is_empty() {
                return;
            }

            let mut head: Option<Box<dyn ComplexEvent>> = None;
            let mut tail = &mut head;

            for mut e in expired_batch {
                e.set_event_type(ComplexEventType::Expired);
                e.set_timestamp(timestamp);
                *tail = Some(Box::new(e.clone_without_next()));
                tail = tail.as_mut().unwrap().mut_next_ref_option();
            }

            for e in &current_batch {
                *tail = Some(Box::new(e.clone_without_next()));
                tail = tail.as_mut().unwrap().mut_next_ref_option();
            }

            {
                let mut ex = self.expired.lock().unwrap();
                ex.extend(current_batch);
            }

            if let Some(chain) = head {
                next.lock().unwrap().process(Some(chain));
            }
        }
        *self.start_time.lock().unwrap() = Some(timestamp);
        if let Some(ref scheduler) = self.scheduler {
            let task = BatchFlushTask {
                buffer: Arc::clone(&self.buffer),
                expired: Arc::clone(&self.expired),
                next: self.meta.next_processor.as_ref().map(Arc::clone),
                duration_ms: self.duration_ms,
                scheduler: Arc::clone(scheduler),
                start_time: Arc::clone(&self.start_time),
            };
            scheduler.notify_at(timestamp + self.duration_ms, Arc::new(task));
        }
    }
}

impl Processor for TimeBatchWindowProcessor {
    fn process(&self, complex_event_chunk: Option<Box<dyn ComplexEvent>>) {
        if let Some(chunk) = complex_event_chunk {
            let mut current_opt = Some(chunk.as_ref() as &dyn ComplexEvent);
            while let Some(ev) = current_opt {
                if let Some(se) = ev.as_any().downcast_ref::<StreamEvent>() {
                    let mut start = self.start_time.lock().unwrap();
                    if start.is_none() {
                        *start = Some(se.timestamp);
                        if let Some(ref scheduler) = self.scheduler {
                            let task = BatchFlushTask {
                                buffer: Arc::clone(&self.buffer),
                                expired: Arc::clone(&self.expired),
                                next: self.meta.next_processor.as_ref().map(Arc::clone),
                                duration_ms: self.duration_ms,
                                scheduler: Arc::clone(scheduler),
                                start_time: Arc::clone(&self.start_time),
                            };
                            scheduler.notify_at(se.timestamp + self.duration_ms, Arc::new(task));
                        }
                    } else if se.timestamp - start.unwrap() >= self.duration_ms {
                        let ts = se.timestamp;
                        drop(start);
                        self.flush(ts);
                    }
                    self.buffer.lock().unwrap().push(se.clone_without_next());
                }
                current_opt = ev.get_next();
            }
        }
    }

    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        self.meta.next_processor.as_ref().map(Arc::clone)
    }

    fn set_next_processor(&mut self, next: Option<Arc<Mutex<dyn Processor>>>) {
        self.meta.next_processor = next;
    }

    fn clone_processor(&self, ctx: &Arc<SiddhiQueryContext>) -> Box<dyn Processor> {
        Box::new(Self::new(
            self.duration_ms,
            Arc::clone(&self.meta.siddhi_app_context),
            Arc::clone(ctx),
        ))
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

impl WindowProcessor for TimeBatchWindowProcessor {}

#[derive(Debug, Clone)]
pub struct TimeBatchWindowFactory;

impl WindowProcessorFactory for TimeBatchWindowFactory {
    fn name(&self) -> &'static str {
        "timeBatch"
    }
    fn create(
        &self,
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Arc<Mutex<dyn Processor>>, String> {
        Ok(Arc::new(Mutex::new(
            TimeBatchWindowProcessor::from_handler(handler, app_ctx, query_ctx)?,
        )))
    }

    fn clone_box(&self) -> Box<dyn WindowProcessorFactory> {
        Box::new(Self {})
    }
}

// ---- ExternalTimeWindowProcessor ----

#[derive(Debug)]
pub struct ExternalTimeWindowProcessor {
    meta: CommonProcessorMeta,
    pub duration_ms: i64,
    buffer: Arc<Mutex<VecDeque<Arc<StreamEvent>>>>,
}

impl ExternalTimeWindowProcessor {
    pub fn new(
        duration_ms: i64,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Self {
        Self {
            meta: CommonProcessorMeta::new(app_ctx, query_ctx),
            duration_ms,
            buffer: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn from_handler(
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Self, String> {
        let expr = handler
            .get_parameters()
            .get(1)
            .ok_or("externalTime window requires a duration parameter")?;
        if let Expression::Constant(c) = expr {
            let dur = match &c.value {
                ConstantValueWithFloat::Time(t) => *t,
                ConstantValueWithFloat::Long(l) => *l,
                _ => return Err("externalTime window duration must be time constant".to_string()),
            };
            Ok(Self::new(dur, app_ctx, query_ctx))
        } else {
            Err("externalTime window duration must be constant".to_string())
        }
    }
}

impl Processor for ExternalTimeWindowProcessor {
    fn process(&self, complex_event_chunk: Option<Box<dyn ComplexEvent>>) {
        if let Some(ref next) = self.meta.next_processor {
            if let Some(chunk) = complex_event_chunk {
                let mut current_opt = Some(chunk.as_ref() as &dyn ComplexEvent);
                while let Some(ev) = current_opt {
                    if let Some(se) = ev.as_any().downcast_ref::<StreamEvent>() {
                        let ts = se.timestamp;
                        let mut expired_head: Option<Box<dyn ComplexEvent>> = None;
                        let mut tail = &mut expired_head;
                        {
                            let mut buf = self.buffer.lock().unwrap();
                            while let Some(front) = buf.front() {
                                if ts - front.timestamp >= self.duration_ms {
                                    if let Some(old) = buf.pop_front() {
                                        let mut ex = old.as_ref().clone_without_next();
                                        ex.set_event_type(ComplexEventType::Expired);
                                        ex.set_timestamp(ts);
                                        *tail = Some(Box::new(ex));
                                        tail = tail.as_mut().unwrap().mut_next_ref_option();
                                    }
                                } else {
                                    break;
                                }
                            }
                            buf.push_back(Arc::new(se.clone_without_next()));
                        }

                        *tail = Some(Box::new(se.clone_without_next()));
                        next.lock().unwrap().process(expired_head);
                    }
                    current_opt = ev.get_next();
                }
            }
        }
    }

    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        self.meta.next_processor.as_ref().map(Arc::clone)
    }

    fn set_next_processor(&mut self, next: Option<Arc<Mutex<dyn Processor>>>) {
        self.meta.next_processor = next;
    }

    fn clone_processor(&self, ctx: &Arc<SiddhiQueryContext>) -> Box<dyn Processor> {
        Box::new(Self::new(
            self.duration_ms,
            Arc::clone(&self.meta.siddhi_app_context),
            Arc::clone(ctx),
        ))
    }

    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> {
        Arc::clone(&self.meta.siddhi_app_context)
    }

    fn get_processing_mode(&self) -> ProcessingMode {
        ProcessingMode::SLIDE
    }

    fn is_stateful(&self) -> bool {
        true
    }
}

impl WindowProcessor for ExternalTimeWindowProcessor {}

#[derive(Debug, Clone)]
pub struct ExternalTimeWindowFactory;

impl WindowProcessorFactory for ExternalTimeWindowFactory {
    fn name(&self) -> &'static str {
        "externalTime"
    }
    fn create(
        &self,
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Arc<Mutex<dyn Processor>>, String> {
        Ok(Arc::new(Mutex::new(
            ExternalTimeWindowProcessor::from_handler(handler, app_ctx, query_ctx)?,
        )))
    }

    fn clone_box(&self) -> Box<dyn WindowProcessorFactory> {
        Box::new(Self {})
    }
}

// ---- ExternalTimeBatchWindowProcessor ----

#[derive(Debug)]
pub struct ExternalTimeBatchWindowProcessor {
    meta: CommonProcessorMeta,
    pub duration_ms: i64,
    buffer: Arc<Mutex<Vec<StreamEvent>>>,
    expired: Arc<Mutex<Vec<StreamEvent>>>,
    start_time: Arc<Mutex<Option<i64>>>,
}

impl ExternalTimeBatchWindowProcessor {
    pub fn new(
        duration_ms: i64,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Self {
        Self {
            meta: CommonProcessorMeta::new(app_ctx, query_ctx),
            duration_ms,
            buffer: Arc::new(Mutex::new(Vec::new())),
            expired: Arc::new(Mutex::new(Vec::new())),
            start_time: Arc::new(Mutex::new(None)),
        }
    }

    pub fn from_handler(
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Self, String> {
        let expr = handler
            .get_parameters()
            .get(1)
            .ok_or("externalTimeBatch window requires a duration parameter")?;
        if let Expression::Constant(c) = expr {
            let dur = match &c.value {
                ConstantValueWithFloat::Time(t) => *t,
                ConstantValueWithFloat::Long(l) => *l,
                _ => {
                    return Err(
                        "externalTimeBatch window duration must be time constant".to_string()
                    )
                }
            };
            Ok(Self::new(dur, app_ctx, query_ctx))
        } else {
            Err("externalTimeBatch window duration must be constant".to_string())
        }
    }

    fn flush(&self, timestamp: i64) {
        if let Some(ref next) = self.meta.next_processor {
            let expired_batch: Vec<StreamEvent> = {
                let mut ex = self.expired.lock().unwrap();
                std::mem::take(&mut *ex)
            };
            let current_batch: Vec<StreamEvent> = {
                let mut buf = self.buffer.lock().unwrap();
                std::mem::take(&mut *buf)
            };

            if expired_batch.is_empty() && current_batch.is_empty() {
                return;
            }

            let mut head: Option<Box<dyn ComplexEvent>> = None;
            let mut tail = &mut head;

            for mut e in expired_batch {
                e.set_event_type(ComplexEventType::Expired);
                e.set_timestamp(timestamp);
                *tail = Some(Box::new(e.clone_without_next()));
                tail = tail.as_mut().unwrap().mut_next_ref_option();
            }

            for e in &current_batch {
                *tail = Some(Box::new(e.clone_without_next()));
                tail = tail.as_mut().unwrap().mut_next_ref_option();
            }

            {
                let mut ex = self.expired.lock().unwrap();
                ex.extend(current_batch);
            }

            if let Some(chain) = head {
                next.lock().unwrap().process(Some(chain));
            }
        }
        *self.start_time.lock().unwrap() = Some(timestamp);
    }
}

impl Processor for ExternalTimeBatchWindowProcessor {
    fn process(&self, complex_event_chunk: Option<Box<dyn ComplexEvent>>) {
        if let Some(chunk) = complex_event_chunk {
            let mut current_opt = Some(chunk.as_ref() as &dyn ComplexEvent);
            while let Some(ev) = current_opt {
                if let Some(se) = ev.as_any().downcast_ref::<StreamEvent>() {
                    let ts = se.timestamp;
                    let mut start = self.start_time.lock().unwrap();
                    if start.is_none() {
                        *start = Some(ts);
                    }
                    while ts - start.unwrap() >= self.duration_ms {
                        let flush_ts = start.unwrap() + self.duration_ms;
                        drop(start);
                        self.flush(flush_ts);
                        start = self.start_time.lock().unwrap();
                    }
                    self.buffer.lock().unwrap().push(se.clone_without_next());
                }
                current_opt = ev.get_next();
            }
        }
    }

    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        self.meta.next_processor.as_ref().map(Arc::clone)
    }

    fn set_next_processor(&mut self, next: Option<Arc<Mutex<dyn Processor>>>) {
        self.meta.next_processor = next;
    }

    fn clone_processor(&self, ctx: &Arc<SiddhiQueryContext>) -> Box<dyn Processor> {
        Box::new(Self::new(
            self.duration_ms,
            Arc::clone(&self.meta.siddhi_app_context),
            Arc::clone(ctx),
        ))
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

impl WindowProcessor for ExternalTimeBatchWindowProcessor {}

// ---- LossyCountingWindowProcessor ----

#[derive(Debug)]
pub struct LossyCountingWindowProcessor {
    meta: CommonProcessorMeta,
}

impl LossyCountingWindowProcessor {
    pub fn new(app_ctx: Arc<SiddhiAppContext>, query_ctx: Arc<SiddhiQueryContext>) -> Self {
        Self {
            meta: CommonProcessorMeta::new(app_ctx, query_ctx),
        }
    }

    pub fn from_handler(
        _handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Self, String> {
        Ok(Self::new(app_ctx, query_ctx))
    }
}

impl Processor for LossyCountingWindowProcessor {
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

    fn clone_processor(&self, ctx: &Arc<SiddhiQueryContext>) -> Box<dyn Processor> {
        Box::new(Self::new(
            Arc::clone(&self.meta.siddhi_app_context),
            Arc::clone(ctx),
        ))
    }

    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> {
        Arc::clone(&self.meta.siddhi_app_context)
    }

    fn get_processing_mode(&self) -> ProcessingMode {
        ProcessingMode::DEFAULT
    }

    fn is_stateful(&self) -> bool {
        false
    }
}

impl WindowProcessor for LossyCountingWindowProcessor {}

#[derive(Debug, Clone)]
pub struct LossyCountingWindowFactory;

impl WindowProcessorFactory for LossyCountingWindowFactory {
    fn name(&self) -> &'static str {
        "lossyCounting"
    }
    fn create(
        &self,
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Arc<Mutex<dyn Processor>>, String> {
        Ok(Arc::new(Mutex::new(
            LossyCountingWindowProcessor::from_handler(handler, app_ctx, query_ctx)?,
        )))
    }

    fn clone_box(&self) -> Box<dyn WindowProcessorFactory> {
        Box::new(Self {})
    }
}

// ---- CronWindowProcessor ----

#[derive(Debug)]
pub struct CronWindowProcessor {
    meta: CommonProcessorMeta,
    cron: String,
    scheduler: Option<Arc<Scheduler>>,
    buffer: Arc<Mutex<Vec<StreamEvent>>>,
    expired: Arc<Mutex<Vec<StreamEvent>>>,
    scheduled: Arc<Mutex<bool>>,
}

impl CronWindowProcessor {
    pub fn new(cron: String, app_ctx: Arc<SiddhiAppContext>, query_ctx: Arc<SiddhiQueryContext>) -> Self {
        let scheduler = app_ctx.get_scheduler();
        Self {
            meta: CommonProcessorMeta::new(app_ctx, query_ctx),
            cron,
            scheduler,
            buffer: Arc::new(Mutex::new(Vec::new())),
            expired: Arc::new(Mutex::new(Vec::new())),
            scheduled: Arc::new(Mutex::new(false)),
        }
    }

    pub fn from_handler(
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Self, String> {
        let expr = handler
            .get_parameters()
            .first()
            .ok_or("cron window requires a cron expression")?;
        if let Expression::Constant(c) = expr {
            if let ConstantValueWithFloat::String(s) = &c.value {
                Ok(Self::new(s.clone(), app_ctx, query_ctx))
            } else {
                Err("cron expression must be a string".to_string())
            }
        } else {
            Err("cron expression must be constant".to_string())
        }
    }

    fn schedule(&self) {
        if let Some(ref sched) = self.scheduler {
            if !*self.scheduled.lock().unwrap() {
                let task = CronFlushTask {
                    buffer: Arc::clone(&self.buffer),
                    expired: Arc::clone(&self.expired),
                    next: self.meta.next_processor.as_ref().map(Arc::clone),
                };
                let _ = sched.schedule_cron(&self.cron, Arc::new(task), None);
                *self.scheduled.lock().unwrap() = true;
            }
        }
    }
}

#[derive(Clone)]
struct CronFlushTask {
    buffer: Arc<Mutex<Vec<StreamEvent>>>,
    expired: Arc<Mutex<Vec<StreamEvent>>>,
    next: Option<Arc<Mutex<dyn Processor>>>,
}

impl Schedulable for CronFlushTask {
    fn on_time(&self, timestamp: i64) {
        let expired_batch: Vec<StreamEvent> = {
            let mut ex = self.expired.lock().unwrap();
            std::mem::take(&mut *ex)
        };
        let current_batch: Vec<StreamEvent> = {
            let mut buf = self.buffer.lock().unwrap();
            std::mem::take(&mut *buf)
        };

        if expired_batch.is_empty() && current_batch.is_empty() {
            return;
        }

        let mut head: Option<Box<dyn ComplexEvent>> = None;
        let mut tail = &mut head;

        for mut e in expired_batch {
            e.set_event_type(ComplexEventType::Expired);
            e.set_timestamp(timestamp);
            *tail = Some(Box::new(e.clone_without_next()));
            tail = tail.as_mut().unwrap().mut_next_ref_option();
        }

        for e in &current_batch {
            *tail = Some(Box::new(e.clone_without_next()));
            tail = tail.as_mut().unwrap().mut_next_ref_option();
        }

        {
            let mut ex = self.expired.lock().unwrap();
            ex.extend(current_batch);
        }

        if let Some(chain) = head {
            if let Some(ref next) = self.next {
                next.lock().unwrap().process(Some(chain));
            }
        }
    }
}

impl Processor for CronWindowProcessor {
    fn process(&self, complex_event_chunk: Option<Box<dyn ComplexEvent>>) {
        if let Some(chunk) = complex_event_chunk {
            let mut current_opt = Some(chunk.as_ref() as &dyn ComplexEvent);
            while let Some(ev) = current_opt {
                if let Some(se) = ev.as_any().downcast_ref::<StreamEvent>() {
                    self.buffer.lock().unwrap().push(se.clone_without_next());
                }
                current_opt = ev.get_next();
            }
        }
    }

    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        self.meta.next_processor.as_ref().map(Arc::clone)
    }

    fn set_next_processor(&mut self, next: Option<Arc<Mutex<dyn Processor>>>) {
        self.meta.next_processor = next;
        self.schedule();
    }

    fn clone_processor(&self, ctx: &Arc<SiddhiQueryContext>) -> Box<dyn Processor> {
        Box::new(Self::new(
            self.cron.clone(),
            Arc::clone(&self.meta.siddhi_app_context),
            Arc::clone(ctx),
        ))
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

impl WindowProcessor for CronWindowProcessor {}

#[derive(Debug, Clone)]
pub struct CronWindowFactory;

impl WindowProcessorFactory for CronWindowFactory {
    fn name(&self) -> &'static str {
        "cron"
    }
    fn create(
        &self,
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Arc<Mutex<dyn Processor>>, String> {
        Ok(Arc::new(Mutex::new(
            CronWindowProcessor::from_handler(handler, app_ctx, query_ctx)?,
        )))
    }

    fn clone_box(&self) -> Box<dyn WindowProcessorFactory> {
        Box::new(Self {})
    }
}

#[derive(Debug, Clone)]
pub struct ExternalTimeBatchWindowFactory;

impl WindowProcessorFactory for ExternalTimeBatchWindowFactory {
    fn name(&self) -> &'static str {
        "externalTimeBatch"
    }
    fn create(
        &self,
        handler: &WindowHandler,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Result<Arc<Mutex<dyn Processor>>, String> {
        Ok(Arc::new(Mutex::new(
            ExternalTimeBatchWindowProcessor::from_handler(handler, app_ctx, query_ctx)?,
        )))
    }

    fn clone_box(&self) -> Box<dyn WindowProcessorFactory> {
        Box::new(Self {})
    }
}
