use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::event::event::Event;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TableInputHandler {
    pub siddhi_app_context: Arc<SiddhiAppContext>,
    // placeholder for table reference
}

impl TableInputHandler {
    pub fn new(siddhi_app_context: Arc<SiddhiAppContext>) -> Self {
        Self { siddhi_app_context }
    }

    pub fn add(&self, _events: Vec<Event>) {
        // placeholder
    }
}
