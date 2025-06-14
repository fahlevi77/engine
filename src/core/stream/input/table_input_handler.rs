use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::event::event::Event;
use crate::core::event::value::AttributeValue;
use crate::core::table::Table;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TableInputHandler {
    pub siddhi_app_context: Arc<SiddhiAppContext>,
    table: Arc<dyn Table>,
}

impl TableInputHandler {
    pub fn new(table: Arc<dyn Table>, siddhi_app_context: Arc<SiddhiAppContext>) -> Self {
        Self {
            siddhi_app_context,
            table,
        }
    }

    pub fn add(&self, events: Vec<Event>) {
        for event in events {
            self.table.insert(&event.data);
        }
    }

    pub fn update(&self, old: Vec<AttributeValue>, new: Vec<AttributeValue>) -> bool {
        self.table.update(&old, &new)
    }

    pub fn delete(&self, values: Vec<AttributeValue>) -> bool {
        self.table.delete(&values)
    }
}
