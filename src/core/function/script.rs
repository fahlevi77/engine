use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use std::fmt::Debug;
use std::sync::Arc;

pub trait Script: Debug + Send + Sync {
    fn init(&mut self, ctx: &Arc<SiddhiAppContext>) -> Result<(), String>;
    fn eval(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue>;
    fn clone_box(&self) -> Box<dyn Script>;
}

impl Clone for Box<dyn Script> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
