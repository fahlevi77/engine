use crate::query_api::siddhi_element::SiddhiElement;
use super::on_demand_query::{OnDemandQuery, OnDemandQueryType}; // Import Rust OnDemandQuery

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)] // Added Eq, Hash, Copy
pub enum StoreQueryType {
    Insert,
    Delete,
    Update,
    Select,
    UpdateOrInsert,
    Find,
}

impl Default for StoreQueryType {
    fn default() -> Self { StoreQueryType::Select } // Defaulting to Select/Find
}

// Removed: impl From<java_sys::StoreQueryType> for StoreQueryType { ... }
// Removed: mod java_sys { ... }


#[derive(Clone, Debug, PartialEq)] // Default will be custom via new()
pub struct StoreQuery {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement
    pub on_demand_query: OnDemandQuery,
}

impl StoreQuery {
    pub fn new(on_demand_query: OnDemandQuery) -> Self {
        // The siddhi_element for StoreQuery should ideally be distinct or mirror
        // the one from on_demand_query. Java's StoreQuery delegates getQueryContextStartIndex/EndIndex
        // to its onDemandQuery instance.
        StoreQuery {
            siddhi_element: on_demand_query.siddhi_element.clone(), // Clone context from inner query
            on_demand_query,
        }
    }

    // Static factory `query()`
    pub fn query() -> Self {
        Self::new(OnDemandQuery::default()) // Create with a default OnDemandQuery
    }

    // getType and setType methods from Java's StoreQuery
    pub fn get_type(&self) -> Option<StoreQueryType> {
        self.on_demand_query.on_demand_query_type.as_ref().map(|odt| {
            match odt {
                OnDemandQueryType::Insert => StoreQueryType::Insert,
                OnDemandQueryType::Delete => StoreQueryType::Delete,
                OnDemandQueryType::Update => StoreQueryType::Update,
                OnDemandQueryType::Select => StoreQueryType::Select,
                OnDemandQueryType::UpdateOrInsert => StoreQueryType::UpdateOrInsert,
                OnDemandQueryType::Find => StoreQueryType::Find,
            }
        })
    }

    pub fn set_type(mut self, store_query_type: StoreQueryType) -> Self {
        let odq_type = match store_query_type {
            StoreQueryType::Insert => OnDemandQueryType::Insert,
            StoreQueryType::Delete => OnDemandQueryType::Delete,
            StoreQueryType::Update => OnDemandQueryType::Update,
            StoreQueryType::Select => OnDemandQueryType::Select,
            StoreQueryType::UpdateOrInsert => OnDemandQueryType::UpdateOrInsert,
            StoreQueryType::Find => OnDemandQueryType::Find,
        };
        // Propagate siddhi_element if it's being managed independently
        let current_siddhi_element = self.siddhi_element.clone();
        self.on_demand_query = self.on_demand_query.set_type(odq_type);
        self.on_demand_query.siddhi_element = current_siddhi_element; // Ensure inner query shares context
        self
    }
}

impl Default for StoreQuery {
    fn default() -> Self {
        let default_odq = OnDemandQuery::default();
        StoreQuery {
            siddhi_element: default_odq.siddhi_element.clone(),
            on_demand_query: default_odq,
        }
    }
}

// Delegate SiddhiElement methods to the composed siddhi_element field.
// Or, if it should always mirror on_demand_query's element:
// impl SiddhiElementAccess for StoreQuery {
//     fn siddhi_element(&self) -> &SiddhiElement {
//         &self.on_demand_query.siddhi_element
//     }
//     fn siddhi_element_mut(&mut self) -> &mut SiddhiElement {
//         &mut self.on_demand_query.siddhi_element
//     }
// }
// For now, StoreQuery has its own siddhi_element field, initialized from on_demand_query.
// This means it can have its own distinct context if needed, though usually it would match.
