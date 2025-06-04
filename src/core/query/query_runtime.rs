// siddhi_rust/src/core/query/query_runtime.rs
// Corresponds to io.siddhi.core.query.QueryRuntimeImpl
use crate::core::query::processor::Processor; // The Processor trait
use crate::core::stream::stream_junction::StreamJunction; // For input stream junction
use std::sync::{Arc, Mutex};
use std::fmt::Debug;
use crate::query_api::execution::query::Query as ApiQuery;
use crate::core::config::siddhi_query_context::SiddhiQueryContext;

pub trait QueryRuntimeTrait: Debug + Send + Sync {
    fn get_query_id(&self) -> &str;
    fn is_stateful(&self) -> bool;
    fn get_query(&self) -> Option<Arc<ApiQuery>>;
}

// Represents the runtime of a single query.
// In Java, QueryRuntimeImpl has fields for queryName, SiddhiQueryContext,
// StreamJunction (for input), QuerySelector, OutputRateLimiter, and OutputCallback.
// The processor chain is: InputJunction -> QuerySelector -> OutputRateLimiter -> OutputCallback (e.g., InsertIntoStreamProcessor)
#[derive(Debug)]
pub struct QueryRuntime {
    pub query_name: String,
    pub api_query: Option<Arc<ApiQuery>>,
    pub siddhi_query_context: Option<Arc<SiddhiQueryContext>>,
    // The input stream junction this query consumes from.
    // The QueryRuntime itself doesn't directly "own" the input junction,
    // but it needs to be registered with it.
    // Storing it here might be for reference or if it needs to interact with it post-setup.
    // However, Java QueryRuntimeImpl doesn't store the input StreamJunction directly as a field.
    // It's passed to QueryParser, which then sets up the links.
    // The entry point to the query's processor chain is more relevant.
    // input_stream_junction: Arc<Mutex<StreamJunction>>,

    // The first processor in this query's specific processing chain.
    // This could be a FilterProcessor, WindowProcessor, QuerySelector, etc.
    pub processor_chain_head: Option<Arc<Mutex<dyn Processor>>>,

    // Add other fields as per QueryRuntimeImpl:
    // pub siddhi_query_context: Arc<SiddhiQueryContext>,
    // pub query_selector: Option<Arc<Mutex<SelectProcessor>>>, // Or QuerySelector if that's the struct name
    // pub output_rate_limiter: Option<Arc<Mutex<OutputRateLimiterPlaceholder>>>,
    // pub output_callback: Option<Arc<Mutex<dyn StreamCallback>>> // Or specific output processor
    // pub is_batch_table_query: bool,
    // pub is_store_query: bool,
}

impl QueryRuntime {
    // query_name is usually derived from annotations or generated.
    // input_junction is needed by QueryParser to connect the processor chain.
    pub fn new(query_name: String) -> Self {
        Self {
            query_name,
            api_query: None,
            siddhi_query_context: None,
            processor_chain_head: None,
        }
    }

    pub fn new_with_context(
        query_name: String,
        api_query: Arc<ApiQuery>,
        siddhi_query_context: Arc<SiddhiQueryContext>,
    ) -> Self {
        Self {
            query_name,
            api_query: Some(api_query),
            siddhi_query_context: Some(siddhi_query_context),
            processor_chain_head: None,
        }
    }

}

impl QueryRuntimeTrait for QueryRuntime {
    fn get_query_id(&self) -> &str {
        &self.query_name
    }

    fn is_stateful(&self) -> bool {
        self.siddhi_query_context
            .as_ref()
            .map(|ctx| ctx.is_stateful())
            .unwrap_or(false)
    }

    fn get_query(&self) -> Option<Arc<ApiQuery>> {
        self.api_query.as_ref().map(Arc::clone)
    }
}

    // TODO: Implement methods from QueryRuntimeImpl if needed, e.g.,
    // get_query_name(), get_input_handler() (if it has one directly),
    // get_query_selector(), set_output_rate_limiter(), get_output_rate_limiter(),
    // get_output_callback(), set_output_callback(), get_siddhi_query_context(),
    // notify_updater(), get_snapshot(), restore_from_snapshot(),
    // start(), stop().
    // Many of these will involve interacting with the processor chain.
