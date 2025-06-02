// Corresponds to io.siddhi.core.config.SiddhiOnDemandQueryContext
use std::sync::Arc;
use super::siddhi_app_context::SiddhiAppContext;
use super::siddhi_query_context::SiddhiQueryContext; // To compose/delegate

#[derive(Debug, Clone)]
pub struct SiddhiOnDemandQueryContext {
    // In Java, this extends SiddhiQueryContext. We'll use composition.
    pub query_context: SiddhiQueryContext,
    pub on_demand_query_string: String,
}

impl SiddhiOnDemandQueryContext {
    pub fn new(
        siddhi_app_context: Arc<SiddhiAppContext>,
        query_name: String,
        query_string: String
    ) -> Self {
        // Java constructor: super(siddhiAppContext, queryName, null);
        // The 'null' is for partitionId, which SiddhiQueryContext::new handles.
        Self {
            query_context: SiddhiQueryContext::new(siddhi_app_context, query_name, None),
            on_demand_query_string: query_string,
        }
    }

    pub fn get_on_demand_query_string(&self) -> &str {
        &self.on_demand_query_string
    }

    // Delegate other SiddhiQueryContext methods if needed, e.g.:
    // pub fn get_siddhi_app_context(&self) -> Arc<SiddhiAppContext> {
    //     Arc::clone(&self.query_context.siddhi_app_context)
    // }
    // Or provide access via `pub query_context`.
}

// Implement Deref and DerefMut to easily access SiddhiQueryContext fields/methods
use std::ops::{Deref, DerefMut};

impl Deref for SiddhiOnDemandQueryContext {
    type Target = SiddhiQueryContext;
    fn deref(&self) -> &Self::Target {
        &self.query_context
    }
}

impl DerefMut for SiddhiOnDemandQueryContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.query_context
    }
}
