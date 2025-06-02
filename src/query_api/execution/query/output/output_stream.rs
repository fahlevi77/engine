use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression;
use super::stream::{SetAttribute, UpdateSet}; // Using UpdateSet


#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum OutputEventType {
    ExpiredEvents,
    CurrentEvents,
    AllEvents,
    AllRawEvents,
    ExpiredRawEvents,
}

impl Default for OutputEventType {
    fn default() -> Self { OutputEventType::CurrentEvents }
}

// Action Structs: These do not compose SiddhiElement directly.
// The outer OutputStream struct holds the context.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct InsertIntoStreamAction {
    pub target_id: String, // Default::default() for String is empty
    pub is_inner_stream: bool,
    pub is_fault_stream: bool,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ReturnStreamAction {
    // No fields
}

#[derive(Clone, Debug, PartialEq)] // Default removed (due to Expression)
pub struct DeleteStreamAction {
    pub target_id: String,
    pub on_delete_expression: Expression,
}

#[derive(Clone, Debug, PartialEq)] // Default removed (due to Expression, SetAttribute)
pub struct UpdateStreamAction {
    pub target_id: String,
    pub on_update_expression: Expression, // This is the 'ON' condition for the update
    pub update_set_clause: Option<UpdateSet>,      // This holds the SET assignments
}

#[derive(Clone, Debug, PartialEq)] // Default removed (due to Expression, SetAttribute)
pub struct UpdateOrInsertStreamAction {
    pub target_id: String,
    pub on_update_expression: Expression,  // This is the 'ON' condition
    pub update_set_clause: Option<UpdateSet>,       // This holds the SET assignments
}


#[derive(Clone, Debug, PartialEq)]
pub enum OutputStreamAction {
    InsertInto(InsertIntoStreamAction),
    Return(ReturnStreamAction),
    Delete(DeleteStreamAction),
    Update(UpdateStreamAction),
    UpdateOrInsert(UpdateOrInsertStreamAction),
}

impl Default for OutputStreamAction {
    fn default() -> Self {
        OutputStreamAction::Return(ReturnStreamAction::default())
    }
}

#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct OutputStream {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    pub action: OutputStreamAction,
    pub output_event_type: Option<OutputEventType>,
}

impl OutputStream {
    pub fn new(action: OutputStreamAction, initial_event_type: Option<OutputEventType>) -> Self {
        OutputStream {
            siddhi_element: SiddhiElement::default(),
            action,
            // Defaulting to CurrentEvents if None, as per Java logic in Query/OnDemandQuery
            output_event_type: initial_event_type.or(Some(OutputEventType::default())),
        }
    }

    pub fn default_return_stream() -> Self {
        Self {
            siddhi_element: SiddhiElement::default(),
            action: OutputStreamAction::Return(ReturnStreamAction::default()),
            output_event_type: Some(OutputEventType::CurrentEvents),
        }
    }

    pub fn get_target_id(&self) -> Option<&str> {
        match &self.action {
            OutputStreamAction::InsertInto(a) => Some(&a.target_id),
            OutputStreamAction::Delete(a) => Some(&a.target_id),
            OutputStreamAction::Update(a) => Some(&a.target_id),
            OutputStreamAction::UpdateOrInsert(a) => Some(&a.target_id),
            OutputStreamAction::Return(_) => None,
        }
    }

    pub fn get_output_event_type(&self) -> Option<OutputEventType> {
        self.output_event_type
    }

    pub fn set_output_event_type(&mut self, event_type: OutputEventType) {
        self.output_event_type = Some(event_type);
    }

    pub fn set_output_event_type_if_none(&mut self, event_type: OutputEventType) {
        if self.output_event_type.is_none() {
            self.output_event_type = Some(event_type);
        }
    }
}

// SiddhiElement is composed. Access via self.siddhi_element.
// If OutputStream needed to be passed as `dyn SiddhiElement`:
// impl SiddhiElement for OutputStream { ... delegate to self.siddhi_element ... }
// However, the main Query/OnDemandQuery structs compose SiddhiElement themselves,
// and OutputStream is a field. So direct composition and access is likely intended.
