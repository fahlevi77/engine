// siddhi_rust/src/core/event/state/state_event_cloner.rs
// Simplified port of io.siddhi.core.event.state.StateEventCloner
use super::{
    meta_state_event::MetaStateEvent, state_event::StateEvent,
    state_event_factory::StateEventFactory,
};

#[derive(Debug, Clone)]
pub struct StateEventCloner {
    stream_event_size: usize,
    output_data_size: usize,
    state_event_factory: StateEventFactory,
}

impl StateEventCloner {
    pub fn new(meta: &MetaStateEvent, factory: StateEventFactory) -> Self {
        Self {
            stream_event_size: meta.stream_event_count(),
            output_data_size: meta.get_output_data_attributes().len(),
            state_event_factory: factory,
        }
    }

    pub fn copy_state_event(&self, state_event: &StateEvent) -> StateEvent {
        let mut new_event = self.state_event_factory.new_instance();
        if self.output_data_size > 0 {
            if let (Some(src), Some(dest)) = (&state_event.output_data, &mut new_event.output_data)
            {
                for i in 0..self.output_data_size {
                    dest[i] = src[i].clone();
                }
            }
        }
        if self.stream_event_size > 0 {
            for i in 0..self.stream_event_size {
                new_event.stream_events[i] = state_event.stream_events[i].clone();
            }
        }
        new_event.event_type = state_event.event_type;
        new_event.timestamp = state_event.timestamp;
        new_event.id = state_event.id;
        new_event
    }
}
