use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression;

// Placeholder for SetAttribute, which is part of Update and UpdateOrInsert queries
// This should eventually be moved to a more appropriate module if it's a standalone concept,
// or be part of the specific action structs.
#[derive(Clone, Debug, PartialEq)]
pub struct SetAttributePlaceholder {
    pub table_column: String, // Or Variable
    pub value_to_set: Expression,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)] // Added Copy
pub enum OutputEventType {
    ExpiredEvents,
    CurrentEvents,
    AllEvents,
    AllRawEvents,
    ExpiredRawEvents, // Added from Java enum
}

impl Default for OutputEventType {
    fn default() -> Self {
        // As per Java's Query.outputStream default (ReturnStream with CURRENT_EVENTS)
        // and updateOutputEventType logic.
        OutputEventType::CurrentEvents
    }
}

// Structs for each specific output stream action, mirroring Java subclasses of OutputStream
#[derive(Clone, Debug, PartialEq)]
pub struct InsertIntoStreamAction {
    pub target_id: String,
    pub is_inner_stream: bool, // Specific to InsertIntoStream in Java
    pub is_fault_stream: bool, // Specific to InsertIntoStream in Java
    // output_event_type is managed by the outer OutputStream struct
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReturnStreamAction {
    // No specific fields, type itself defines the action.
    // output_event_type is managed by the outer OutputStream struct
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeleteStreamAction {
    pub target_id: String,
    pub on_delete_expression: Expression,
    // output_event_type is managed by the outer OutputStream struct
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateStreamAction {
    pub target_id: String,
    pub on_update_expression: Expression,
    pub update_set_attributes: Option<Vec<SetAttributePlaceholder>>, // Corresponds to UpdateSet
    // output_event_type is managed by the outer OutputStream struct
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateOrInsertStreamAction {
    pub target_id: String,
    pub on_update_expression: Expression,
    pub update_set_attributes: Option<Vec<SetAttributePlaceholder>>, // Corresponds to UpdateSet
    // output_event_type is managed by the outer OutputStream struct
}


// Enum to represent the different types of output actions.
#[derive(Clone, Debug, PartialEq)]
pub enum OutputStreamAction {
    InsertInto(InsertIntoStreamAction),
    Return(ReturnStreamAction),
    Delete(DeleteStreamAction),
    Update(UpdateStreamAction),
    UpdateOrInsert(UpdateOrInsertStreamAction),
}

// The main OutputStream struct, which wraps an action and common properties.
#[derive(Clone, Debug, PartialEq)]
pub struct OutputStream {
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,

    pub action: OutputStreamAction,
    pub output_event_type: Option<OutputEventType>, // Common to all, but can be set by Query logic
}

impl OutputStream {
    // Constructor for a specific action
    pub fn new(action: OutputStreamAction, initial_event_type: Option<OutputEventType>) -> Self {
        OutputStream {
            query_context_start_index: None,
            query_context_end_index: None,
            action,
            output_event_type: initial_event_type,
        }
    }

    // Helper for default Query constructor (ReturnStream with CURRENT_EVENTS)
    pub fn default_return_stream() -> Self {
        Self::new(
            OutputStreamAction::Return(ReturnStreamAction {}),
            Some(OutputEventType::CurrentEvents)
        )
    }

    // Getters and setters from Java's OutputStream
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

impl SiddhiElement for OutputStream {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.query_context_start_index }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.query_context_start_index = index; }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.query_context_end_index }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.query_context_end_index = index; }
}
