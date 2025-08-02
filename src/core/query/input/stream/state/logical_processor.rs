use std::sync::{Arc, Mutex};

use super::sequence_processor::SequenceSide;
use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::config::siddhi_query_context::SiddhiQueryContext;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::stream::{
    stream_event::StreamEvent, stream_event_cloner::StreamEventCloner,
    stream_event_factory::StreamEventFactory,
};
use crate::core::event::value::AttributeValue;
use crate::core::query::processor::{CommonProcessorMeta, ProcessingMode, Processor};

#[derive(Debug, Clone, Copy)]
pub enum LogicalType {
    And,
    Or,
}

#[derive(Debug)]
pub struct LogicalProcessor {
    meta: CommonProcessorMeta,
    pub logical_type: LogicalType,
    pub first_buffer: Vec<StreamEvent>,
    pub second_buffer: Vec<StreamEvent>,
    pub first_attr_count: usize,
    pub second_attr_count: usize,
    pub next_processor: Option<Arc<Mutex<dyn Processor>>>,
    first_cloner: Option<StreamEventCloner>,
    second_cloner: Option<StreamEventCloner>,
    event_factory: StreamEventFactory,
}

impl LogicalProcessor {
    pub fn new(
        logical_type: LogicalType,
        first_attr_count: usize,
        second_attr_count: usize,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Self {
        Self {
            meta: CommonProcessorMeta::new(app_ctx, query_ctx),
            logical_type,
            first_buffer: Vec::new(),
            second_buffer: Vec::new(),
            first_attr_count,
            second_attr_count,
            next_processor: None,
            first_cloner: None,
            second_cloner: None,
            event_factory: StreamEventFactory::new(first_attr_count + second_attr_count, 0, 0),
        }
    }

    fn build_joined_event(
        &self,
        first: Option<&StreamEvent>,
        second: Option<&StreamEvent>,
    ) -> StreamEvent {
        let mut event = self.event_factory.new_instance();
        event.timestamp = second
            .map(|s| s.timestamp)
            .unwrap_or_else(|| first.unwrap().timestamp);
        for i in 0..self.first_attr_count {
            let val = first
                .and_then(|f| f.before_window_data.get(i).cloned())
                .unwrap_or(AttributeValue::Null);
            event.before_window_data[i] = val;
        }
        for j in 0..self.second_attr_count {
            let val = second
                .and_then(|s| s.before_window_data.get(j).cloned())
                .unwrap_or(AttributeValue::Null);
            event.before_window_data[self.first_attr_count + j] = val;
        }
        event
    }

    fn forward(&self, se: StreamEvent) {
        if let Some(ref next) = self.next_processor {
            next.lock().unwrap().process(Some(Box::new(se)));
        }
    }

    fn try_produce(&mut self) {
        match self.logical_type {
            LogicalType::And => {
                while !self.first_buffer.is_empty() && !self.second_buffer.is_empty() {
                    let first = self.first_buffer.remove(0);
                    let second = self.second_buffer.remove(0);
                    let joined = self.build_joined_event(Some(&first), Some(&second));
                    self.forward(joined);
                }
            }
            LogicalType::Or => {
                while !self.first_buffer.is_empty() {
                    let first = self.first_buffer.remove(0);
                    let joined = self.build_joined_event(Some(&first), None);
                    self.forward(joined);
                }
                while !self.second_buffer.is_empty() {
                    let second = self.second_buffer.remove(0);
                    let joined = self.build_joined_event(None, Some(&second));
                    self.forward(joined);
                }
            }
        }
    }

    fn process_event(&mut self, side: SequenceSide, mut chunk: Option<Box<dyn ComplexEvent>>) {
        while let Some(mut ce) = chunk {
            chunk = ce.set_next(None);
            if let Some(se) = ce.as_any().downcast_ref::<StreamEvent>() {
                let cloner = match side {
                    SequenceSide::First => {
                        if self.first_cloner.is_none() {
                            self.first_cloner = Some(StreamEventCloner::from_event(se));
                        }
                        self.first_cloner.as_ref().unwrap()
                    }
                    SequenceSide::Second => {
                        if self.second_cloner.is_none() {
                            self.second_cloner = Some(StreamEventCloner::from_event(se));
                        }
                        self.second_cloner.as_ref().unwrap()
                    }
                };
                let se_clone = cloner.copy_stream_event(se);
                match side {
                    SequenceSide::First => {
                        self.first_buffer.push(se_clone);
                    }
                    SequenceSide::Second => {
                        self.second_buffer.push(se_clone);
                    }
                }
                self.try_produce();
            }
        }
    }

    pub fn create_side_processor(
        self_arc: &Arc<Mutex<Self>>,
        side: SequenceSide,
    ) -> Arc<Mutex<LogicalProcessorSide>> {
        Arc::new(Mutex::new(LogicalProcessorSide {
            parent: Arc::clone(self_arc),
            side,
        }))
    }
}

#[derive(Debug)]
pub struct LogicalProcessorSide {
    parent: Arc<Mutex<LogicalProcessor>>,
    side: SequenceSide,
}

impl Processor for LogicalProcessorSide {
    fn process(&self, chunk: Option<Box<dyn ComplexEvent>>) {
        self.parent.lock().unwrap().process_event(self.side, chunk);
    }

    fn next_processor(&self) -> Option<Arc<Mutex<dyn Processor>>> {
        self.parent.lock().unwrap().next_processor.clone()
    }

    fn set_next_processor(&mut self, next: Option<Arc<Mutex<dyn Processor>>>) {
        self.parent.lock().unwrap().next_processor = next;
    }

    fn clone_processor(&self, ctx: &Arc<SiddhiQueryContext>) -> Box<dyn Processor> {
        let parent = self.parent.lock().unwrap();
        let cloned = LogicalProcessor::new(
            parent.logical_type,
            parent.first_attr_count,
            parent.second_attr_count,
            Arc::clone(&parent.meta.siddhi_app_context),
            Arc::clone(ctx),
        );
        let arc = Arc::new(Mutex::new(cloned));
        Box::new(LogicalProcessorSide {
            parent: arc,
            side: self.side,
        })
    }

    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> {
        self.parent.lock().unwrap().meta.siddhi_app_context.clone()
    }

    fn get_siddhi_query_context(&self) -> Arc<SiddhiQueryContext> {
        self.parent.lock().unwrap().meta.get_siddhi_query_context()
    }

    fn get_processing_mode(&self) -> ProcessingMode {
        ProcessingMode::DEFAULT
    }

    fn is_stateful(&self) -> bool {
        true
    }
}
