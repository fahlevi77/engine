use crate::query_api::siddhi_element::SiddhiElement;
use super::output_attribute::OutputAttribute;
use super::order_by_attribute::{OrderByAttribute, Order as OrderByOrder};
use crate::query_api::expression::Variable;
use crate::query_api::expression::Expression;
use crate::query_api::expression::constant::{Constant, ConstantValueWithFloat as ConstantValue};


#[derive(Clone, Debug, PartialEq)] // Default is implemented manually
pub struct Selector {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // Selector fields
    pub selection_list: Vec<OutputAttribute>,
    pub group_by_list: Vec<Variable>,
    pub having_expression: Option<Expression>,
    pub order_by_list: Vec<OrderByAttribute>,
    pub limit: Option<Constant>,
    pub offset: Option<Constant>,
}

impl Selector {
    pub fn new() -> Self {
        Selector {
            siddhi_element: SiddhiElement::default(),
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
        self.selection_list.push(OutputAttribute::new(Some(rename), expression)); // rename is Option<String>
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

    pub fn limit(mut self, constant: Constant) -> Result<Self, String> {
        match constant.value {
            ConstantValue::Int(_) | ConstantValue::Long(_) => {
                self.limit = Some(constant);
                Ok(self)
            }
            _ => Err("'limit' only supports int or long constants".to_string()),
        }
    }

    pub fn offset(mut self, constant: Constant) -> Result<Self, String> {
        match constant.value {
            ConstantValue::Int(_) | ConstantValue::Long(_) => {
                self.offset = Some(constant);
                Ok(self)
            }
            _ => Err("'offset' only supports int or long constants".to_string()),
        }
    }
}

impl Default for Selector {
    fn default() -> Self {
        Self::new()
    }
}

// SiddhiElement is composed. Access via self.siddhi_element.
