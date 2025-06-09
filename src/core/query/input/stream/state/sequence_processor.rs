use std::sync::{Arc, Mutex};

use crate::core::query::processor::{Processor, CommonProcessorMeta, ProcessingMode};
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::stream::stream_event::StreamEvent;
use crate::core::event::value::AttributeValue;
use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::config::siddhi_query_context::SiddhiQueryContext;

#[derive(Debug, Clone, Copy)]
pub enum SequenceType {
    Pattern,
    Sequence,
}

#[derive(Debug)]
pub struct SequenceProcessor {
    meta: CommonProcessorMeta,
    pub sequence_type: SequenceType,
    pub first_buffer: Vec<StreamEvent>,
    pub second_buffer: Vec<StreamEvent>,
    pub first_attr_count: usize,
    pub second_attr_count: usize,
    pub next_processor: Option<Arc<Mutex<dyn Processor>>>,
}

impl SequenceProcessor {
    pub fn new(
        sequence_type: SequenceType,
        first_attr_count: usize,
        second_attr_count: usize,
        app_ctx: Arc<SiddhiAppContext>,
        query_ctx: Arc<SiddhiQueryContext>,
    ) -> Self {
        Self {
            meta: CommonProcessorMeta::new(app_ctx, query_ctx),
            sequence_type,
            first_buffer: Vec::new(),
            second_buffer: Vec::new(),
            first_attr_count,
            second_attr_count,
            next_processor: None,
        }
    }

    fn build_joined_event(&self, first: &StreamEvent, second: &StreamEvent) -> StreamEvent {
        let mut event = StreamEvent::new(
            second.timestamp,
            self.first_attr_count + self.second_attr_count,
            0,
            0,
        );
        for i in 0..self.first_attr_count {
            let val = first
                .before_window_data
                .get(i)
                .cloned()
                .unwrap_or(AttributeValue::Null);
            event.before_window_data[i] = val;
        }
        for j in 0..self.second_attr_count {
            let val = second
                .before_window_data
                .get(j)
                .cloned()
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

    fn check_and_produce(&mut self) {
        while !self.first_buffer.is_empty() && !self.second_buffer.is_empty() {
            let first = self.first_buffer.remove(0);
            let second = self.second_buffer.remove(0);
            let joined = self.build_joined_event(&first, &second);
            self.forward(joined);
        }
    }

    fn process_event(&mut self, side: SequenceSide, mut chunk: Option<Box<dyn ComplexEvent>>) {
        while let Some(mut ce) = chunk {
            chunk = ce.set_next(None);
            if let Some(se) = ce.as_any().downcast_ref::<StreamEvent>() {
                let se_clone = se.clone_without_next();
                match side {
                    SequenceSide::First => {
                        self.first_buffer.push(se_clone);
                        if matches!(self.sequence_type, SequenceType::Pattern) {
                            self.check_and_produce();
                        }
                    }
                    SequenceSide::Second => {
                        self.second_buffer.push(se_clone);
                        self.check_and_produce();
                    }
                }
            }
        }
    }

    pub fn create_side_processor(self_arc: &Arc<Mutex<Self>>, side: SequenceSide) -> Arc<Mutex<SequenceProcessorSide>> {
        Arc::new(Mutex::new(SequenceProcessorSide {
            parent: Arc::clone(self_arc),
            side,
        }))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SequenceSide {
    First,
    Second,
}

#[derive(Debug)]
pub struct SequenceProcessorSide {
    parent: Arc<Mutex<SequenceProcessor>>,
    side: SequenceSide,
}

impl Processor for SequenceProcessorSide {
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
        let cloned = SequenceProcessor::new(
            parent.sequence_type,
            parent.first_attr_count,
            parent.second_attr_count,
            Arc::clone(&parent.meta.siddhi_app_context),
            Arc::clone(ctx),
        );
        let arc = Arc::new(Mutex::new(cloned));
        let side = SequenceProcessorSide { parent: arc, side: self.side };
        Box::new(side)
    }

    fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> {
        self.parent.lock().unwrap().meta.siddhi_app_context.clone()
    }

    fn get_processing_mode(&self) -> ProcessingMode { ProcessingMode::DEFAULT }

    fn is_stateful(&self) -> bool { true }
}
