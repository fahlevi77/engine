// Corresponds to io.siddhi.query.api.execution.query.input.store.Store
// Extends BasicSingleInputStream and implements InputStore in Java.

use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression;
use crate::query_api::execution::query::input::stream::BasicSingleInputStream;
use crate::query_api::execution::query::input::handler::StreamHandler; // For new_with_id constructor
use super::input_store::InputStoreTrait;
// For on() methods returning these types, which are then wrapped in InputStore enum
// These will need to be refactored to compose SiddhiElement as well.
use super::condition_input_store::ConditionInputStore;
use super::aggregation_input_store::AggregationInputStore;

// Using Within from join_input_stream, assuming it's general enough.
// Ideally, this would be `crate::query_api::aggregation::Within` if defined.
use crate::query_api::execution::query::input::stream::join_input_stream::Within as WithinPlaceholder;


#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct Store {
    // Composes BasicSingleInputStream to inherit its stream-like properties
    pub basic_single_input_stream: BasicSingleInputStream,
    // Store has its own SiddhiElement context, separate from BasicSingleInputStream's inner one.
    // However, Java Store directly uses context of BasicSingleInputStream.
    // Let's ensure `siddhi_element` is part of `Store` directly or consistently accessed.
    // The current impl delegates SiddhiElement to basic_single_input_stream.
    // If Store needs its *own* context distinct from the stream it represents,
    // it would need its own `siddhi_element: SiddhiElement` field.
    // For now, assuming it shares context with its BasicSingleInputStream representation.
    // This means Store does not need its own siddhi_element field if BasicSingleInputStream has one.
    // Let's verify BasicSingleInputStream: it has `pub inner: SingleInputStream`, and SingleInputStream has `siddhi_element`.
    // So, Store effectively gets its context via `basic_single_input_stream.inner.siddhi_element`.
    // The SiddhiElement impl for Store already delegates to basic_single_input_stream.
    pub siddhi_element: SiddhiElement, // Store itself is a SiddhiElement
}

impl Store {
    // Internal constructor, used by factories in InputStore enum.
    // Made pub(super) in previous version, now pub for direct use if needed.
    pub fn new_with_id(store_id: String) -> Self {
        Store {
            siddhi_element: SiddhiElement::default(),
            basic_single_input_stream: BasicSingleInputStream::new(store_id, false, false, None, Vec::new()),
        }
    }

    pub fn new_with_ref(store_reference_id: String, store_id: String) -> Self {
        Store {
            siddhi_element: SiddhiElement::default(),
            basic_single_input_stream: BasicSingleInputStream::new(store_id, false, false, Some(store_reference_id), Vec::new()),
        }
    }

    // Constructor from prompt: `new(store_id: String, on_condition: Option<Expression>)`
    // This seems to conflate Store with ConditionInputStore.
    // Java Store is just an ID; conditions are applied via `.on()`.
    // Sticking to Java structure: Store itself doesn't take on_condition in constructor.

    // Methods corresponding to `on(Expression)` and `on(Within, Per)`
    // These return concrete types which can then be wrapped by the InputStore enum.
    pub fn on_condition(self, on_condition: Expression) -> ConditionInputStore {
        // ConditionInputStore would take `self` (a Store instance) and the condition.
        ConditionInputStore::new(self, on_condition) // Assumes ConditionInputStore::new is defined
    }

    pub fn on_aggregation_condition(
        self,
        on_condition: Expression,
        within: WithinPlaceholder,
        per: Expression
    ) -> AggregationInputStore {
        AggregationInputStore::new_with_condition(self, on_condition, within, per)  // Assumes constructor exists
    }

    pub fn on_aggregation_only(
        self,
        within: WithinPlaceholder,
        per: Expression
    ) -> AggregationInputStore {
        AggregationInputStore::new_no_condition(self, within, per) // Assumes constructor exists
    }
}

// Implement SiddhiElement for Store.
// It should have its own context, separate from the BasicSingleInputStream it composes if it's a distinct element.
// Java's Store extends BasicSingleInputStream, so it *is* a BasicSingleInputStream and shares its context.
// Our BasicSingleInputStream gets context via its `inner: SingleInputStream`.
// The previous SiddhiElement impl for Store delegated to basic_single_input_stream.
// If Store itself is a SiddhiElement, it should use its own `siddhi_element` field.
// The prompt's example for `StoreQuery` has `element: SiddhiElement`, implying StoreQuery is the element,
// not necessarily the `Store` struct itself if `Store` is just an ID wrapper.
// Let's assume `Store` itself needs to be a `SiddhiElement`.

impl SiddhiElement for Store {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.siddhi_element.query_context_start_index }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.siddhi_element.query_context_start_index = index; }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.siddhi_element.query_context_end_index }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.siddhi_element.query_context_end_index = index; }
}

// Implement InputStoreTrait
impl InputStoreTrait for Store {
    fn get_store_id(&self) -> &str {
        // The ID of the store is the stream_id of the composed BasicSingleInputStream
        self.basic_single_input_stream.inner.get_stream_id_str()
    }

    fn get_store_reference_id(&self) -> Option<&str> {
        self.basic_single_input_stream.inner.get_stream_reference_id_str()
    }
}

// Delegate stream-like methods (filter, function, window, as) to BasicSingleInputStream
impl Store {
     pub fn filter(mut self, filter_expression: Expression) -> Self {
        self.basic_single_input_stream = self.basic_single_input_stream.filter(filter_expression);
        self
    }
    // TODO: Add other delegated methods: function, window, as_ref from BasicSingleInputStream
}
