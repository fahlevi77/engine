use crate::query_api::siddhi_element::SiddhiElement;
use crate::query_api::expression::Variable;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)] // Added Eq, Hash, Copy
pub enum Order {
    Asc,
    Desc,
}

impl Default for Order {
    fn default() -> Self { Order::Asc }
}


#[derive(Clone, Debug, PartialEq)] // Default requires Variable to be Default
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
impl Default for OrderByAttribute {
    fn default() -> Self {
        Self {
            siddhi_element: SiddhiElement::default(),
            variable: Variable::default(),
            order: Order::default(),
        }
    }
}
