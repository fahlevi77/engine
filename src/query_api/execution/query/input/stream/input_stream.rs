// Corresponds to io.siddhi.query.api.execution.query.input.stream.InputStream (abstract class)
use crate::query_api::siddhi_element::SiddhiElement;

// Import specific stream types that will be variants of the InputStream enum
use super::single_input_stream::SingleInputStream;
use super::join_input_stream::{JoinInputStream, Type as JoinType, EventTrigger as JoinEventTrigger};
use super::state_input_stream::{StateInputStream, Type as StateInputStreamType};
use super::basic_single_input_stream::BasicSingleInputStream; // For factory methods
use super::anonymous_input_stream::AnonymousInputStream; // For factory methods

// For factory methods that need these:
use crate::query_api::execution::query::input::state::StateElement;
use crate::query_api::expression::Expression;
use crate::query_api::expression::constant::Constant as ExpressionConstant;
// use crate::query_api::aggregation::Within; // TODO: Define if Within is used by JoinInputStream factories
type WithinPlaceholder = String; // Placeholder


// Trait for common methods of InputStream (from Java's InputStream abstract class)
pub trait InputStreamTrait { // Removed : SiddhiElement as SiddhiElement is directly implemented by the enum
    fn get_all_stream_ids(&self) -> Vec<String>;
    fn get_unique_stream_ids(&self) -> Vec<String>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputStream {
    Single(SingleInputStream),
    Join(Box<JoinInputStream>),
    State(Box<StateInputStream>),
}

// SiddhiElement implementation for the InputStream enum
impl InputStream {
    // These helpers assume that SingleInputStream, JoinInputStream, StateInputStream
    // will be refactored to compose `siddhi_element: SiddhiElement`.
    fn siddhi_element_ref(&self) -> &SiddhiElement {
        match self {
            InputStream::Single(s) => &s.siddhi_element,
            InputStream::Join(j) => &j.siddhi_element,
            InputStream::State(st) => &st.siddhi_element,
        }
    }

    fn siddhi_element_mut_ref(&mut self) -> &mut SiddhiElement {
        match self {
            InputStream::Single(s) => &mut s.siddhi_element,
            InputStream::Join(j) => &mut j.siddhi_element,
            InputStream::State(st) => &mut st.siddhi_element,
        }
    }
}

impl SiddhiElement for InputStream {
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

impl InputStreamTrait for InputStream {
    fn get_all_stream_ids(&self) -> Vec<String> {
        match self {
            InputStream::Single(s) => s.get_all_stream_ids(), // Assumes SingleInputStream impls InputStreamTrait
            InputStream::Join(j) => j.get_all_stream_ids(),   // Assumes JoinInputStream impls InputStreamTrait
            InputStream::State(st) => st.get_all_stream_ids(), // Assumes StateInputStream impls InputStreamTrait
        }
    }

    fn get_unique_stream_ids(&self) -> Vec<String> {
        match self {
            InputStream::Single(s) => s.get_unique_stream_ids(),
            InputStream::Join(j) => j.get_unique_stream_ids(),
            InputStream::State(st) => st.get_unique_stream_ids(),
        }
    }
}

// Factory methods from Java's InputStream abstract class
impl InputStream {
    // Basic streams
    pub fn stream(stream_id: String) -> Self {
        InputStream::Single(SingleInputStream::new_basic_from_id(stream_id))
    }
    pub fn stream_with_ref(stream_reference_id: String, stream_id: String) -> Self {
        InputStream::Single(SingleInputStream::new_basic_from_ref_id(stream_reference_id, stream_id))
    }
    pub fn inner_stream(stream_id: String) -> Self {
        InputStream::Single(SingleInputStream::new_basic_inner_from_id(stream_id))
    }
    // TODO: Add other BasicSingleInputStream factories (inner_with_ref, fault_stream, etc.)

    // Anonymous stream from Query
    pub fn stream_from_query(query: crate::query_api::execution::query::Query) -> Self {
        InputStream::Single(SingleInputStream::new_anonymous_from_query(query))
    }

    // Join streams
    pub fn join_stream(
        left_stream: SingleInputStream,
        join_type: JoinType,
        right_stream: SingleInputStream,
        on_compare: Option<Expression>,
        trigger: Option<JoinEventTrigger>, // Made Option, Java defaults to ALL
        within: Option<WithinPlaceholder>, // TODO: Replace with actual Within type
        per: Option<Expression>
    ) -> Self {
        InputStream::Join(Box::new(JoinInputStream::new(
            left_stream,
            join_type,
            right_stream,
            on_compare,
            trigger.unwrap_or(JoinEventTrigger::All), // Default from Java
            within,
            per,
        )))
    }
    // TODO: Add other overloaded JoinInputStream factories from Java

    // Pattern streams
    pub fn pattern_stream(pattern_element: StateElement, within_time: Option<ExpressionConstant>) -> Self {
        InputStream::State(Box::new(StateInputStream::pattern_stream(pattern_element, within_time)))
    }

    // Sequence streams
    pub fn sequence_stream(sequence_element: StateElement, within_time: Option<ExpressionConstant>) -> Self {
        InputStream::State(Box::new(StateInputStream::sequence_stream(sequence_element, within_time)))
    }
}
