// SPDX-License-Identifier: MIT OR Apache-2.0

// Corresponds to io.eventflux.core.config.EventFluxOnDemandQueryContext
use super::eventflux_app_context::EventFluxAppContext;
use super::eventflux_query_context::EventFluxQueryContext;
use std::sync::Arc; // To compose/delegate

#[derive(Debug, Clone)]
pub struct EventFluxOnDemandQueryContext {
    // In Java, this extends EventFluxQueryContext. We'll use composition.
    pub query_context: EventFluxQueryContext,
    pub on_demand_query_string: String,
}

impl EventFluxOnDemandQueryContext {
    pub fn new(
        eventflux_app_context: Arc<EventFluxAppContext>,
        query_name: String,
        query_string: String,
    ) -> Self {
        // Java constructor: super(eventfluxAppContext, queryName, null);
        // The 'null' is for partitionId, which EventFluxQueryContext::new handles.
        Self {
            query_context: EventFluxQueryContext::new(eventflux_app_context, query_name, None),
            on_demand_query_string: query_string,
        }
    }

    pub fn get_on_demand_query_string(&self) -> &str {
        &self.on_demand_query_string
    }

    // Delegate other EventFluxQueryContext methods if needed, e.g.:
    // pub fn get_eventflux_app_context(&self) -> Arc<EventFluxAppContext> {
    //     Arc::clone(&self.query_context.eventflux_app_context)
    // }
    // Or provide access via `pub query_context`.
}

// Implement Deref and DerefMut to easily access EventFluxQueryContext fields/methods
use std::ops::{Deref, DerefMut};

impl Deref for EventFluxOnDemandQueryContext {
    type Target = EventFluxQueryContext;
    fn deref(&self) -> &Self::Target {
        &self.query_context
    }
}

impl DerefMut for EventFluxOnDemandQueryContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.query_context
    }
}
