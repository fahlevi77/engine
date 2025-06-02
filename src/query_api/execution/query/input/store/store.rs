// Corresponds to io.siddhi.query.api.execution.query.input.store.Store
// Extends BasicSingleInputStream and implements InputStore in Java.

use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression;
// BasicSingleInputStream for composition/delegation
use crate::query_api::execution::query::input::stream::BasicSingleInputStream;
// For InputStoreTrait
use super::input_store::InputStoreTrait;
// For on() methods returning these types, which are then wrapped in InputStore enum
use super::condition_input_store::ConditionInputStore;
use super::aggregation_input_store::AggregationInputStore;

// Placeholders for Within and Per, as they are complex types not yet fully defined.
// use crate::query_api::aggregation::Within; // TODO: Define this
// use crate::query_api::expression::Expression as PerExpression; // TODO: Confirm 'per' type
type WithinPlaceholder = String;
type PerExpressionPlaceholder = Expression;


#[derive(Clone, Debug, PartialEq)]
pub struct Store {
    // Composes BasicSingleInputStream to inherit its stream-like properties
    // (stream_id, stream_reference_id, handlers, etc.)
    pub basic_single_input_stream: BasicSingleInputStream,
    // Store doesn't add new fields other than those inherited from BasicSingleInputStream
    // and methods from InputStore.
}

impl Store {
    // Constructors from Java (protected, but used by InputStore.store() static methods)
    pub(super) fn new_with_id(store_id: String) -> Self {
        Store {
            // Create a BasicSingleInputStream with the store_id. No reference_id by default.
            // No handlers initially for a plain store reference.
            basic_single_input_stream: BasicSingleInputStream::new(store_id, false, false, None, Vec::new()),
        }
    }

    pub(super) fn new_with_ref(store_reference_id: String, store_id: String) -> Self {
        Store {
            basic_single_input_stream: BasicSingleInputStream::new(store_id, false, false, Some(store_reference_id), Vec::new()),
        }
    }

    // Methods corresponding to `on(Expression)` and `on(Within, Per)`
    // These return concrete types which can then be wrapped by the InputStore enum.
    pub fn on_condition(self, on_condition: Expression) -> ConditionInputStore {
        ConditionInputStore::new(self, on_condition)
    }

    pub fn on_aggregation_condition(
        self,
        on_condition: Expression,
        within: WithinPlaceholder, // TODO: Replace with actual Within type
        per: PerExpressionPlaceholder // TODO: Replace with actual Per type
    ) -> AggregationInputStore {
        AggregationInputStore::new_with_condition(self, on_condition, within, per)
    }

    pub fn on_aggregation_only(
        self,
        within: WithinPlaceholder, // TODO: Replace with actual Within type
        per: PerExpressionPlaceholder // TODO: Replace with actual Per type
    ) -> AggregationInputStore {
        AggregationInputStore::new_no_condition(self, within, per)
    }

}

// Implement SiddhiElement by delegating to BasicSingleInputStream
impl SiddhiElement for Store {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.basic_single_input_stream.query_context_start_index() }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.basic_single_input_stream.set_query_context_start_index(index); }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.basic_single_input_stream.query_context_end_index() }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.basic_single_input_stream.set_query_context_end_index(index); }
}

// Implement InputStoreTrait
impl InputStoreTrait for Store {
    fn get_store_id(&self) -> &str {
        self.basic_single_input_stream.inner.get_stream_id_str()
    }

    fn get_store_reference_id(&self) -> Option<&str> {
        self.basic_single_input_stream.inner.get_stream_reference_id_str()
    }
}

// Delegate stream-like methods (filter, function, window, as) to BasicSingleInputStream
// This allows Store to be used like a stream before specifying `on` conditions.
impl Store {
     pub fn filter(mut self, filter_expression: Expression) -> Self {
        self.basic_single_input_stream = self.basic_single_input_stream.filter(filter_expression);
        self
    }

    // Add other delegated methods from BasicSingleInputStream as needed (function, window, as_ref)
}
