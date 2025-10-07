// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::core::config::eventflux_app_context::EventFluxAppContext;
use crate::core::event::event::Event;
use crate::core::event::value::AttributeValue;
use crate::core::table::{InMemoryCompiledCondition, InMemoryCompiledUpdateSet, Table};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TableInputHandler {
    pub eventflux_app_context: Arc<EventFluxAppContext>,
    table: Arc<dyn Table>,
}

impl TableInputHandler {
    pub fn new(table: Arc<dyn Table>, eventflux_app_context: Arc<EventFluxAppContext>) -> Self {
        Self {
            eventflux_app_context,
            table,
        }
    }

    pub fn add(&self, events: Vec<Event>) {
        for event in events {
            self.table.insert(&event.data);
        }
    }

    pub fn update(&self, old: Vec<AttributeValue>, new: Vec<AttributeValue>) -> bool {
        let cond = InMemoryCompiledCondition { values: old };
        let us = InMemoryCompiledUpdateSet { values: new };
        self.table.update(&cond, &us)
    }

    pub fn delete(&self, values: Vec<AttributeValue>) -> bool {
        let cond = InMemoryCompiledCondition { values };
        self.table.delete(&cond)
    }
}
