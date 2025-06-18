use crate::core::event::event::Event;
use std::fmt::Debug;

pub trait SourceMapper: Debug + Send + Sync {
    fn map(&self, input: &[u8]) -> Vec<Event>;
    fn clone_box(&self) -> Box<dyn SourceMapper>;
}

impl Clone for Box<dyn SourceMapper> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
