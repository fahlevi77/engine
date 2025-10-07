// SPDX-License-Identifier: MIT OR Apache-2.0

// Corresponds to io.eventflux.query.api.execution.query.input.state.StateElement (interface)
use crate::query_api::eventflux_element::EventFluxElement;

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

// Implement EventFluxElement for the enum by dispatching to variants' composed eventflux_element field
impl StateElement {
    fn eventflux_element_ref(&self) -> &EventFluxElement {
        match self {
            StateElement::Stream(s) => &s.eventflux_element,
            StateElement::AbsentStream(a) => a.eventflux_element(),
            StateElement::Logical(l) => &l.eventflux_element,
            StateElement::Next(n) => &n.eventflux_element,
            StateElement::Count(c) => &c.eventflux_element,
            StateElement::Every(e) => &e.eventflux_element,
        }
    }

    fn eventflux_element_mut_ref(&mut self) -> &mut EventFluxElement {
        match self {
            StateElement::Stream(s) => &mut s.eventflux_element,
            StateElement::AbsentStream(a) => a.eventflux_element_mut(),
            StateElement::Logical(l) => &mut l.eventflux_element,
            StateElement::Next(n) => &mut n.eventflux_element,
            StateElement::Count(c) => &mut c.eventflux_element,
            StateElement::Every(e) => &mut e.eventflux_element,
        }
    }
}

// `impl EventFluxElement for StateElement` removed.

// Removed placeholder: impl StreamStateElement { pub(crate) fn get_stream_id_placeholder(&self) -> String }
// This functionality should be part of StreamStateElement's own API if needed by other modules.
