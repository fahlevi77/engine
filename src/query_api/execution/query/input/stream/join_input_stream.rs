// Corresponds to io.siddhi.query.api.execution.query.input.stream.JoinInputStream
use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression;
use super::single_input_stream::SingleInputStream;
use super::input_stream::InputStreamTrait;
use crate::query_api::aggregation::Within; // Using the actual Within struct


#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)] // Added Eq, Hash, Copy
pub enum Type {
    Join,
    InnerJoin,
    LeftOuterJoin,
    RightOuterJoin,
    FullOuterJoin,
}

impl Default for Type {
    fn default() -> Self { Type::Join }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)] // Added Eq, Hash, Copy
pub enum EventTrigger {
    Left,
    Right,
    All,
}

impl Default for EventTrigger {
    fn default() -> Self { EventTrigger::All }
}

#[derive(Clone, Debug, PartialEq)] // Default will be custom
pub struct JoinInputStream {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    pub left_input_stream: Box<SingleInputStream>,
    pub join_type: Type,
    pub right_input_stream: Box<SingleInputStream>,
    pub on_compare: Option<Expression>,

    pub trigger: EventTrigger,
    pub within: Option<Within>, // Using the actual Within struct
    pub per: Option<Expression>,
}

impl JoinInputStream {
    pub fn new(
        left_input_stream: SingleInputStream,
        join_type: Type,
        right_input_stream: SingleInputStream,
        on_compare: Option<Expression>,
        trigger: EventTrigger,
        within: Option<Within>, // Using the actual Within struct
        per: Option<Expression>,
    ) -> Self {
        JoinInputStream {
            siddhi_element: SiddhiElement::default(),
            left_input_stream: Box::new(left_input_stream),
            join_type,
            right_input_stream: Box::new(right_input_stream),
            on_compare,
            trigger,
            within,
            per,
        }
    }
}

impl Default for JoinInputStream {
    fn default() -> Self {
        JoinInputStream {
            siddhi_element: SiddhiElement::default(),
            // Defaulting to Box::default() which is Box::new(SingleInputStream::default())
            left_input_stream: Box::default(),
            join_type: Type::default(),
            right_input_stream: Box::default(),
            on_compare: None,
            trigger: EventTrigger::default(),
            within: None,
            per: None,
        }
    }
}


impl InputStreamTrait for JoinInputStream {
    fn get_all_stream_ids(&self) -> Vec<String> {
        let mut ids = self.left_input_stream.get_all_stream_ids();
        ids.extend(self.right_input_stream.get_all_stream_ids());
        ids
    }

    fn get_unique_stream_ids(&self) -> Vec<String> {
        let mut ids = self.left_input_stream.get_unique_stream_ids();
        for id in self.right_input_stream.get_unique_stream_ids() {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
        ids
    }
}
