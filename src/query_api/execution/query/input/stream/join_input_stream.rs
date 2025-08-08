// Corresponds to io.siddhi.query.api.execution.query.input.stream.JoinInputStream
use super::input_stream::InputStreamTrait;
use super::single_input_stream::SingleInputStream;
use crate::query_api::aggregation::Within;
use crate::query_api::expression::Expression;
use crate::query_api::siddhi_element::SiddhiElement; // Using the actual Within struct

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)] // Added Eq, Hash, Copy
#[derive(Default)]
pub enum Type {
    #[default]
    Join,
    InnerJoin,
    LeftOuterJoin,
    RightOuterJoin,
    FullOuterJoin,
}


#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)] // Added Eq, Hash, Copy
#[derive(Default)]
pub enum EventTrigger {
    Left,
    Right,
    #[default]
    All,
}


#[derive(Clone, Debug, PartialEq)] // Default will be custom
#[derive(Default)]
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
