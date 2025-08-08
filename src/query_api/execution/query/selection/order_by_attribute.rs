use crate::query_api::expression::Variable;
use crate::query_api::siddhi_element::SiddhiElement;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)] // Added Eq, Hash, Copy
#[derive(Default)]
pub enum Order {
    #[default]
    Asc,
    Desc,
}


#[derive(Clone, Debug, PartialEq)] // Default requires Variable to be Default
#[derive(Default)]
pub struct OrderByAttribute {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement

    // OrderByAttribute fields
    pub variable: Variable, // Variable struct has Default derive
    pub order: Order,
}

impl OrderByAttribute {
    // Constructor for `OrderByAttribute(Variable variable, Order order)`
    pub fn new(variable: Variable, order: Order) -> Self {
        OrderByAttribute {
            siddhi_element: SiddhiElement::default(), // Or copy from variable.siddhi_element?
            variable,
            order,
        }
    }

    // Constructor for `OrderByAttribute(Variable variable)` which defaults to ASC
    pub fn new_default_order(variable: Variable) -> Self {
        Self::new(variable, Order::default())
    }

    pub fn get_variable(&self) -> &Variable {
        &self.variable
    }

    pub fn get_order(&self) -> &Order {
        &self.order
    }
}

// If Variable is Default, OrderByAttribute can derive Default.
// Variable's Default requires attribute_name to be String::default().
