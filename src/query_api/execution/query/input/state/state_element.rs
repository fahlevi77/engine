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
    AbsentStream(AbsentStreamStateElement),
    Logical(LogicalStateElement), // Contains Box<StateElement> internally for its operands
    Next(Box<NextStateElement>),
    Count(CountStateElement), // Contains Box<StreamStateElement> internally
    Every(Box<EveryStateElement>),
}

// Implement SiddhiElement for the enum by dispatching to variants' composed siddhi_element field
impl StateElement {
    fn siddhi_element_ref(&self) -> &SiddhiElement {
        match self {
            StateElement::Stream(s) => &s.siddhi_element,
            StateElement::AbsentStream(a) => &a.siddhi_element,
            StateElement::Logical(l) => &l.siddhi_element,
            StateElement::Next(n) => &n.siddhi_element,
            StateElement::Count(c) => &c.siddhi_element,
            StateElement::Every(e) => &e.siddhi_element,
        }
    }

    fn siddhi_element_mut_ref(&mut self) -> &mut SiddhiElement {
        match self {
            StateElement::Stream(s) => &mut s.siddhi_element,
            StateElement::AbsentStream(a) => &mut a.siddhi_element,
            StateElement::Logical(l) => &mut l.siddhi_element,
            StateElement::Next(n) => &mut n.siddhi_element,
            StateElement::Count(c) => &mut c.siddhi_element,
            StateElement::Every(e) => &mut e.siddhi_element,
        }
    }
}

impl SiddhiElement for StateElement {
    fn query_context_start_index(&self) -> Option<(i32, i32)> {
        self.siddhi_element_ref().query_context_start_index
    }

    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) {
        self.siddhi_element_mut_ref().query_context_start_index = index;
    }

    fn query_context_end_index(&self) -> Option<(i32, i32)> {
        self.siddhi_element_ref().query_context_end_index
    }

    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) {
        self.siddhi_element_mut_ref().query_context_end_index = index;
    }
}

// Removed placeholder: impl StreamStateElement { pub(crate) fn get_stream_id_placeholder(&self) -> String }
// This functionality should be part of StreamStateElement's own API if needed by other modules.
