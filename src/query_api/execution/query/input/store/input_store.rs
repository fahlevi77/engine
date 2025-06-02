// Corresponds to io.siddhi.query.api.execution.query.input.store.InputStore (interface)
use crate::query_api::siddhi_element::SiddhiElement; // All InputStores are SiddhiElements implicitly or explicitly

// Forward declare specific store types
use super::store::Store;
use super::condition_input_store::ConditionInputStore;
use super::aggregation_input_store::AggregationInputStore;


// Trait for common InputStore behavior
pub trait InputStoreTrait: SiddhiElement {
    fn get_store_id(&self) -> &str;
    fn get_store_reference_id(&self) -> Option<&str>;
    // Add other common methods if any. For now, these are the main ones from the interface.
}

// Enum to concretely represent different types of InputStores,
// useful for fields like OnDemandQuery.inputStore
#[derive(Clone, Debug, PartialEq)]
pub enum InputStore {
    Store(Store), // Direct table/aggregation/window
    Condition(ConditionInputStore), // Store with an ON condition
    Aggregation(AggregationInputStore), // Store with WITHIN and PER for aggregation
}

impl InputStore {
    // Static factory methods from Java's InputStore interface
    pub fn store(store_id: String) -> Store {
        Store::new_with_id(store_id)
    }

    pub fn store_with_ref(store_reference_id: String, store_id: String) -> Store {
        Store::new_with_ref(store_reference_id, store_id)
    }

    // Helper to access SiddhiElement, dispatching to variants
    pub fn siddhi_element(&self) -> &dyn SiddhiElement {
        match self {
            InputStore::Store(s) => s, // Store itself needs to implement SiddhiElement
            InputStore::Condition(c) => c, // ConditionInputStore needs to implement SiddhiElement
            InputStore::Aggregation(a) => a, // AggregationInputStore needs to implement SiddhiElement
        }
    }
    pub fn siddhi_element_mut(&mut self) -> &mut dyn SiddhiElement {
        match self {
            InputStore::Store(s) => s,
            InputStore::Condition(c) => c,
            InputStore::Aggregation(a) => a,
        }
    }
}

// Implement SiddhiElement for the enum
impl SiddhiElement for InputStore {
    fn query_context_start_index(&self) -> Option<(i32, i32)> {
        self.siddhi_element().query_context_start_index()
    }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) {
        self.siddhi_element_mut().set_query_context_start_index(index);
    }
    fn query_context_end_index(&self) -> Option<(i32, i32)> {
        self.siddhi_element().query_context_end_index()
    }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) {
        self.siddhi_element_mut().set_query_context_end_index(index);
    }
}

// Implement InputStoreTrait for the enum
impl InputStoreTrait for InputStore {
    fn get_store_id(&self) -> &str {
        match self {
            InputStore::Store(s) => s.get_store_id(),
            InputStore::Condition(c) => c.get_store_id(),
            InputStore::Aggregation(a) => a.get_store_id(),
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

// Individual structs (Store, ConditionInputStore, AggregationInputStore)
// will also need to implement InputStoreTrait and SiddhiElement.
