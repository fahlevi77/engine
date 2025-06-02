use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Expression;
use crate::query_api::expression::Variable; // For the constructor OutputAttribute(Variable)

#[derive(Clone, Debug, PartialEq)]
pub struct OutputAttribute {
    // SiddhiElement fields
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,

    // OutputAttribute fields
    pub rename: Option<String>, // In Java, rename is String, but can be null if created from Variable
    pub expression: Expression,
}

impl OutputAttribute {
    // Constructor for `OutputAttribute(String rename, Expression expression)`
    pub fn new(rename: String, expression: Expression) -> Self {
        OutputAttribute {
            query_context_start_index: None,
            query_context_end_index: None,
            rename: Some(rename),
            expression,
        }
    }

    // Constructor for `OutputAttribute(Variable variable)`
    pub fn new_from_variable(variable: Variable) -> Self {
        OutputAttribute {
            query_context_start_index: None, // Variable itself will have context
            query_context_end_index: None,
            rename: Some(variable.attribute_name.clone()), // `rename` is the attribute name
            expression: Expression::Variable(variable), // Expression is the variable itself
        }
    }
}

impl SiddhiElement for OutputAttribute {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.query_context_start_index }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.query_context_start_index = index; }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.query_context_end_index }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.query_context_end_index = index; }
}
