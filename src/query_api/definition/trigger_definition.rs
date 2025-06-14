// Corresponds to io.siddhi.query.api.definition.TriggerDefinition
use crate::query_api::expression::constant::{Constant, ConstantValueWithFloat};
use crate::query_api::siddhi_element::SiddhiElement;

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct TriggerDefinition {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // TriggerDefinition fields
    pub id: String,
    pub at_every: Option<i64>, // Java Long maps to i64 in Rust
    pub at: Option<String>,    // For cron expressions or similar string-based time definitions
}

impl TriggerDefinition {
    // Constructor that takes an id, as per Java's `TriggerDefinition.id(id)`
    pub fn new(id: String) -> Self {
        TriggerDefinition {
            siddhi_element: SiddhiElement::default(),
            id,
            at_every: None,
            at: None,
        }
    }

    // Static factory method `id` from Java
    pub fn id(id: String) -> Self {
        // This creates a new TriggerDefinition with the given ID and defaults for other fields
        TriggerDefinition {
            id,
            ..Default::default()
        }
    }

    // Builder-style methods
    pub fn at_every(mut self, time_in_milliseconds: i64) -> Self {
        self.at_every = Some(time_in_milliseconds);
        self
    }

    // Builder method for TimeConstant
    pub fn at_every_time_constant(mut self, time_constant: Constant) -> Result<Self, String> {
        match time_constant.value {
            ConstantValueWithFloat::Time(ms) => {
                self.at_every = Some(ms);
                Ok(self)
            }
            ConstantValueWithFloat::Long(ms) => {
                // Also allow Long for flexibility as in Java
                self.at_every = Some(ms);
                Ok(self)
            }
            _ => Err("at_every_time_constant expects a Time or Long constant".to_string()),
        }
    }

    pub fn at(mut self, interval: String) -> Self {
        self.at = Some(interval);
        self
    }
}

// Removed: impl From<SiddhiElement> for TriggerDefinition

// For direct field access via Deref, if desired:
// impl std::ops::Deref for TriggerDefinition {
//     type Target = SiddhiElement;
//     fn deref(&self) -> &Self::Target { &self.siddhi_element }
// }
// impl std::ops::DerefMut for TriggerDefinition {
//     fn deref_mut(&mut self) -> &mut Self::Target { &mut self.siddhi_element }
// }
