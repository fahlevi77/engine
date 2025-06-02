use crate::query_api::siddhi_element::SiddhiElement;
use super::output_attribute::OutputAttribute;
use super::order_by_attribute::{OrderByAttribute, Order as OrderByOrder}; // Renamed to avoid conflict
use crate::query_api::expression::Variable; // Corrected path
use crate::query_api::expression::Expression;
use crate::query_api::expression::constant::Constant; // Path to the unified Constant struct

#[derive(Clone, Debug, PartialEq)]
pub struct Selector {
    // SiddhiElement fields
    pub query_context_start_index: Option<(i32, i32)>,
    pub query_context_end_index: Option<(i32, i32)>,

    // Selector fields
    pub selection_list: Vec<OutputAttribute>,
    pub group_by_list: Vec<Variable>,
    pub having_expression: Option<Expression>,
    pub order_by_list: Vec<OrderByAttribute>,
    pub limit: Option<Constant>,
    pub offset: Option<Constant>,
}

impl Selector {
    // Default constructor, matches `new Selector()` in Java
    pub fn new() -> Self {
        Selector {
            query_context_start_index: None,
            query_context_end_index: None,
            selection_list: Vec::new(),
            group_by_list: Vec::new(),
            having_expression: None,
            order_by_list: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    // Static factory `selector()`
    pub fn selector() -> Self {
        Self::new()
    }

    // Builder methods
    pub fn select(mut self, rename: String, expression: Expression) -> Self {
        // TODO: Add checkSelection logic from Java (duplicate rename check)
        self.selection_list.push(OutputAttribute::new(rename, expression));
        self
    }

    pub fn select_variable(mut self, variable: Variable) -> Self {
        // TODO: Add checkSelection logic
        self.selection_list.push(OutputAttribute::new_from_variable(variable));
        self
    }

    pub fn add_selection_list(mut self, mut projection_list: Vec<OutputAttribute>) -> Self {
        // TODO: Add checkSelection logic for each item
        self.selection_list.append(&mut projection_list);
        self
    }

    pub fn group_by(mut self, variable: Variable) -> Self {
        self.group_by_list.push(variable);
        self
    }

    pub fn add_group_by_list(mut self, mut list: Vec<Variable>) -> Self {
        self.group_by_list.append(&mut list);
        self
    }

    pub fn having(mut self, having_expression: Expression) -> Self {
        self.having_expression = Some(having_expression);
        self
    }

    pub fn order_by(mut self, variable: Variable) -> Self {
        self.order_by_list.push(OrderByAttribute::new_default_order(variable));
        self
    }

    pub fn order_by_with_order(mut self, variable: Variable, order: OrderByOrder) -> Self {
        self.order_by_list.push(OrderByAttribute::new(variable, order));
        self
    }

    pub fn add_order_by_list(mut self, mut list: Vec<OrderByAttribute>) -> Self {
        self.order_by_list.append(&mut list);
        self
    }

    pub fn limit(mut self, constant: Constant) -> Self {
        // TODO: Add type check from Java (IntConstant or LongConstant)
        // This requires ConstantValue enum to be accessible.
        // match constant.value {
        //     ConstantValue::Int(_) | ConstantValue::Long(_) => self.limit = Some(constant),
        //     _ => panic!("'limit' only supports int or long constants"),
        // }
        self.limit = Some(constant); // Simplified for now
        self
    }

    pub fn offset(mut self, constant: Constant) -> Self {
        // TODO: Add type check similar to limit
        self.offset = Some(constant); // Simplified for now
        self
    }
}

impl Default for Selector {
    fn default() -> Self {
        Self::new()
    }
}

impl SiddhiElement for Selector {
    fn query_context_start_index(&self) -> Option<(i32,i32)> { self.query_context_start_index }
    fn set_query_context_start_index(&mut self, index: Option<(i32,i32)>) { self.query_context_start_index = index; }
    fn query_context_end_index(&self) -> Option<(i32,i32)> { self.query_context_end_index }
    fn set_query_context_end_index(&mut self, index: Option<(i32,i32)>) { self.query_context_end_index = index; }
}
