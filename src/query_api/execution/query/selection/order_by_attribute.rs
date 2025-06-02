use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Variable; // Corrected path

#[derive(Clone, Debug, PartialEq)]
pub enum Order {
    Asc,
    Desc,
}

// Helper to convert from Java enum string if needed during parsing, not used now.
// impl FromStr for Order { ... }


#[derive(Clone, Debug, PartialEq)]
pub struct OrderByAttribute {
    // SiddhiElement fields
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,

    // OrderByAttribute fields
    pub variable: Variable,
    pub order: Order, // Default is ASC in Java
}

impl OrderByAttribute {
    // Constructor for `OrderByAttribute(Variable variable, Order order)`
    pub fn new(variable: Variable, order: Order) -> Self {
        OrderByAttribute {
            query_context_start_index: None, // Variable itself will have context
            query_context_end_index: None,
            variable,
            order,
        }
    }

    // Constructor for `OrderByAttribute(Variable variable)` which defaults to ASC
    pub fn new_default_order(variable: Variable) -> Self {
        Self::new(variable, Order::Asc)
    }
}

impl SiddhiElement for OrderByAttribute {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.query_context_start_index }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.query_context_start_index = index; }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.query_context_end_index }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.query_context_end_index = index; }
}
