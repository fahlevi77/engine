// Corresponds to io.siddhi.query.api.execution.query.input.store.ConditionInputStore
use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression;
use super::store::Store; // The Store struct (wrapper around BasicSingleInputStream)
use super::input_store::InputStoreTrait; // To implement get_store_id etc.

#[derive(Clone, Debug, PartialEq)] // Default is not straightforward due to Store and Expression
pub struct ConditionInputStore {
    pub siddhi_element: SiddhiElement, // For its own context if needed

    pub store: Store, // The underlying store (table, window, aggregation)
    pub on_condition: Option<Expression>, // Condition is optional in some Store.on() calls in Java, but required for ConditionInputStore
}

impl ConditionInputStore {
    pub fn new(store: Store, on_condition: Expression) -> Self {
        ConditionInputStore {
            siddhi_element: SiddhiElement::default(), // Or copy context from store?
            store,
            on_condition: Some(on_condition),
        }
    }

    // If on_condition can truly be optional for this type (Java constructor implies it's not)
    // pub fn new_optional_condition(store: Store, on_condition: Option<Expression>) -> Self { ... }
}

impl SiddhiElement for ConditionInputStore {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.siddhi_element.query_context_start_index }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.siddhi_element.query_context_start_index = index; }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.siddhi_element.query_context_end_index }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.siddhi_element.query_context_end_index = index; }
}

impl InputStoreTrait for ConditionInputStore {
    fn get_store_id(&self) -> &str {
        self.store.get_store_id() // Delegate to composed Store
    }

    fn get_store_reference_id(&self) -> Option<&str> {
        self.store.get_store_reference_id() // Delegate to composed Store
    }
}
