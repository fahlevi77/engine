// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::core::config::eventflux_app_context::EventFluxAppContext;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use std::fmt::Debug;
use std::sync::Arc;

pub trait Script: Debug + Send + Sync {
    fn init(&mut self, ctx: &Arc<EventFluxAppContext>) -> Result<(), String>;
    fn eval(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue>;
    fn clone_box(&self) -> Box<dyn Script>;
}

impl Clone for Box<dyn Script> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
