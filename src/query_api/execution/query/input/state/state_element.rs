// Corresponds to io.siddhi.query.api.execution.query.input.state.StateElement (interface)
use crate::query_api::siddhi_element::SiddhiElement;

// Import specific state element types
use super::absent_stream_state_element::AbsentStreamStateElement;
use super::count_state_element::CountStateElement;
use super::every_state_element::EveryStateElement;
use super::logical_state_element::LogicalStateElement;
use super::next_state_element::NextStateElement;
use super::stream_state_element::StreamStateElement;


#[derive(Clone, Debug, PartialEq)]
pub enum StateElement {
    Stream(StreamStateElement),
    AbsentStream(AbsentStreamStateElement), // Derived from logicalNot in State.java
    Logical(LogicalStateElement),
    Next(Box<NextStateElement>),     // Boxed for recursive definition
    Count(CountStateElement),        // CountStateElement itself contains a StreamStateElement
    Every(Box<EveryStateElement>),   // Boxed for recursive definition
}

// Implement SiddhiElement for the enum by dispatching to variants
impl SiddhiElement for StateElement {
    fn query_context_start_index(&self) -> Option<(i32, i32)> {
        match self {
            StateElement::Stream(s) => s.query_context_start_index(),
            StateElement::AbsentStream(a) => a.query_context_start_index(),
            StateElement::Logical(l) => l.query_context_start_index(),
            StateElement::Next(n) => n.query_context_start_index(),
            StateElement::Count(c) => c.query_context_start_index(),
            StateElement::Every(e) => e.query_context_start_index(),
        }
    }

    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) {
        match self {
            StateElement::Stream(s) => s.set_query_context_start_index(index),
            StateElement::AbsentStream(a) => a.set_query_context_start_index(index),
            StateElement::Logical(l) => l.set_query_context_start_index(index),
            StateElement::Next(n) => n.set_query_context_start_index(index),
            StateElement::Count(c) => c.set_query_context_start_index(index),
            StateElement::Every(e) => e.set_query_context_start_index(index),
        }
    }

    fn query_context_end_index(&self) -> Option<(i32, i32)> {
        match self {
            StateElement::Stream(s) => s.query_context_end_index(),
            StateElement::AbsentStream(a) => a.query_context_end_index(),
            StateElement::Logical(l) => l.query_context_end_index(),
            StateElement::Next(n) => n.query_context_end_index(),
            StateElement::Count(c) => c.query_context_end_index(),
            StateElement::Every(e) => e.query_context_end_index(),
        }
    }

    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) {
        match self {
            StateElement::Stream(s) => s.set_query_context_end_index(index),
            StateElement::AbsentStream(a) => a.set_query_context_end_index(index),
            StateElement::Logical(l) => l.set_query_context_end_index(index),
            StateElement::Next(n) => n.set_query_context_end_index(index),
            StateElement::Count(c) => c.set_query_context_end_index(index),
            StateElement::Every(e) => e.set_query_context_end_index(index),
        }
    }
}

// Placeholder method for StreamStateElement to use in StateInputStream
// This is a bit of a hack due to module organization. Ideally, StreamStateElement would provide this.
impl StreamStateElement {
    pub(crate) fn get_stream_id_placeholder(&self) -> String {
        // This needs access to BasicSingleInputStream's stream_id
        // For now, returning a placeholder.
        // In a real implementation, this would properly access the ID.
        self.basic_single_input_stream.inner.get_stream_id_str().to_string()
    }
}
