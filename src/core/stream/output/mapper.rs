use crate::core::event::event::Event;
use std::fmt::Debug;

pub trait SinkMapper: Debug + Send + Sync {
    fn map(&self, events: &[Event]) -> Vec<u8>;
    fn clone_box(&self) -> Box<dyn SinkMapper>;
}

impl Clone for Box<dyn SinkMapper> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
