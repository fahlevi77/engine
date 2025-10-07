// Corresponds to io.eventflux.query.api.execution.query.input.store.ConditionInputStore
use super::input_store::InputStoreTrait;
use super::store::Store; // The Store struct (wrapper around BasicSingleInputStream)
use crate::query_api::eventflux_element::EventFluxElement;
use crate::query_api::expression::Expression; // To implement get_store_id etc.

#[derive(Clone, Debug, PartialEq)] // Default is not straightforward due to Store and Expression
pub struct ConditionInputStore {
    pub eventflux_element: EventFluxElement, // For its own context if needed

    pub store: Store, // The underlying store (table, window, aggregation)
    pub on_condition: Option<Expression>, // Condition is optional in some Store.on() calls in Java, but required for ConditionInputStore
}

impl ConditionInputStore {
    pub fn new(store: Store, on_condition: Expression) -> Self {
        ConditionInputStore {
            eventflux_element: EventFluxElement::default(), // Or copy context from store?
            store,
            on_condition: Some(on_condition),
        }
    }

    // If on_condition can truly be optional for this type (Java constructor implies it's not)
    // pub fn new_optional_condition(store: Store, on_condition: Option<Expression>) -> Self { ... }
}

// `impl EventFluxElement for ConditionInputStore` removed.

impl InputStoreTrait for ConditionInputStore {
    fn get_store_id(&self) -> &str {
        self.store.get_store_id() // Delegate to composed Store
    }

    fn get_store_reference_id(&self) -> Option<&str> {
        self.store.get_store_reference_id() // Delegate to composed Store
    }
}
