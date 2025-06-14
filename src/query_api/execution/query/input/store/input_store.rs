// Corresponds to io.siddhi.query.api.execution.query.input.store.InputStore (interface)
use crate::query_api::siddhi_element::SiddhiElement;

// Import specific store types that will be variants of the InputStore enum
use super::store::Store;
// ConditionInputStore and AggregationInputStore are not yet refactored in this pass.
// Assuming they will be. For now, their direct usage might be problematic if their structure changed.
use super::aggregation_input_store::AggregationInputStore;
use super::condition_input_store::ConditionInputStore;

// Trait for common InputStore behavior
pub trait InputStoreTrait {
    // Removed SiddhiElement supertrait
    fn get_store_id(&self) -> &str;
    fn get_store_reference_id(&self) -> Option<&str>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputStore {
    Store(Store),
    Condition(Box<ConditionInputStore>), // Box if these can be large or recursive
    Aggregation(Box<AggregationInputStore>),
}

impl InputStore {
    // Static factory methods from Java's InputStore interface for creating Store instances
    // These return Store, which can then be wrapped into InputStore::Store if needed.
    pub fn store(store_id: String) -> Store {
        Store::new_with_id(store_id) // Assuming Store::new_with_id exists
    }

    pub fn store_with_ref(store_reference_id: String, store_id: String) -> Store {
        Store::new_with_ref(store_reference_id, store_id) // Assuming Store::new_with_ref exists
    }

    // Helper to access the composed siddhi_element from variants
    fn siddhi_element_ref(&self) -> &SiddhiElement {
        match self {
            InputStore::Store(s) => &s.siddhi_element,
            InputStore::Condition(c) => &c.siddhi_element,
            InputStore::Aggregation(a) => &a.siddhi_element,
        }
    }
    fn siddhi_element_mut_ref(&mut self) -> &mut SiddhiElement {
        match self {
            InputStore::Store(s) => &mut s.siddhi_element,
            InputStore::Condition(c) => &mut c.siddhi_element,
            InputStore::Aggregation(a) => &mut a.siddhi_element,
        }
    }
}

// `impl SiddhiElement for InputStore` removed.

// Implement InputStoreTrait for the enum
impl InputStoreTrait for InputStore {
    fn get_store_id(&self) -> &str {
        match self {
            InputStore::Store(s) => s.get_store_id(), // Assumes Store impls InputStoreTrait
            InputStore::Condition(c) => c.get_store_id(), // Assumes ConditionInputStore impls InputStoreTrait
            InputStore::Aggregation(a) => a.get_store_id(), // Assumes AggregationInputStore impls InputStoreTrait
        }
    }

    fn get_store_reference_id(&self) -> Option<&str> {
        match self {
            InputStore::Store(s) => s.get_store_reference_id(),
            InputStore::Condition(c) => c.get_store_reference_id(),
            InputStore::Aggregation(a) => a.get_store_reference_id(),
        }
    }
}

// Note: The structs Store, ConditionInputStore, AggregationInputStore
// must be refactored to:
// 1. Compose `siddhi_element: SiddhiElement`.
// 2. Implement `InputStoreTrait`.
// ConditionInputStore and AggregationInputStore were not part of this subtask's explicit file list to create/refactor,
// but they are used by this enum. They were created in subtask 0005. Their review is pending.
// Boxing Condition and Aggregation variants as they compose other types and might be larger.
