use super::set_attribute::SetAttribute;
use crate::query_api::eventflux_element::EventFluxElement;
use crate::query_api::expression::Expression;
use crate::query_api::expression::Variable;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct UpdateSet {
    pub eventflux_element: EventFluxElement, // UpdateSet in Java is a EventFluxElement
    pub set_attributes: Vec<SetAttribute>,
}

impl UpdateSet {
    pub fn new() -> Self {
        Self {
            eventflux_element: EventFluxElement::default(),
            set_attributes: Vec::new(),
        }
    }

    // Corresponds to Java's `set(Variable tableVariable, Expression assignmentExpression)`
    pub fn add_set_attribute(mut self, table_column: Variable, value_to_set: Expression) -> Self {
        self.set_attributes
            .push(SetAttribute::new(table_column, value_to_set));
        self
    }

    // The prompt suggested `on_set_condition: Option<Expression>`.
    // In Java, `UpdateSet` does not hold this condition. The condition is part of
    // `UpdateStream` or `UpdateOrInsertStream` (passed as `onUpdateExpression`).
    // So, this field is omitted here.
}
