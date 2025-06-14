use super::order_by_attribute::{Order as OrderByOrder, OrderByAttribute};
use super::output_attribute::OutputAttribute;
use crate::query_api::expression::constant::{Constant, ConstantValueWithFloat as ConstantValue};
use crate::query_api::expression::Expression;
use crate::query_api::expression::Variable;
use crate::query_api::siddhi_element::SiddhiElement;

/// Handles the SELECT part of a query, including projections, group by, etc.
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
        self.selection_list
            .push(OutputAttribute::new(Some(rename), expression)); // rename is Option<String>
        self
    }

    pub fn select_variable(mut self, variable: Variable) -> Self {
        // TODO: Add checkSelection logic
        self.selection_list
            .push(OutputAttribute::new_from_variable(variable));
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
        self.order_by_list
            .push(OrderByAttribute::new_default_order(variable));
        self
    }

    pub fn order_by_with_order(mut self, variable: Variable, order: OrderByOrder) -> Self {
        self.order_by_list
            .push(OrderByAttribute::new(variable, order));
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

    // Getter methods
    pub fn get_selection_list(&self) -> &Vec<OutputAttribute> {
        &self.selection_list
    }

    pub fn get_group_by_list(&self) -> &Vec<Variable> {
        &self.group_by_list
    }

    pub fn get_having_expression(&self) -> Option<&Expression> {
        self.having_expression.as_ref()
    }

    pub fn get_order_by_list(&self) -> &Vec<OrderByAttribute> {
        &self.order_by_list
    }

    pub fn get_limit(&self) -> Option<&Constant> {
        self.limit.as_ref()
    }

    pub fn get_offset(&self) -> Option<&Constant> {
        self.offset.as_ref()
    }
}

impl Default for Selector {
    fn default() -> Self {
        Self::new()
    }
}

// SiddhiElement is composed. Access via self.siddhi_element.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query_api::execution::query::selection::order_by_attribute::{
        Order as OrderByOrder, OrderByAttribute,
    };
    use crate::query_api::execution::query::selection::output_attribute::OutputAttribute;
    use crate::query_api::expression::constant::{Constant, ConstantValueWithFloat};
    use crate::query_api::expression::variable::Variable;
    use crate::query_api::expression::Expression;

    #[test]
    fn test_selector_new() {
        let s = Selector::new();
        assert!(s.get_selection_list().is_empty());
        assert!(s.get_group_by_list().is_empty());
        assert!(s.get_having_expression().is_none());
        assert!(s.get_order_by_list().is_empty());
        assert!(s.get_limit().is_none());
        assert!(s.get_offset().is_none());
        assert_eq!(s.siddhi_element.query_context_start_index, None);
    }

    #[test]
    fn test_selector_select() {
        let expr = Expression::Constant(Constant::string("val".to_string()));
        let s = Selector::selector().select("renamed".to_string(), expr.clone());
        assert_eq!(s.get_selection_list().len(), 1);
        assert_eq!(
            s.get_selection_list()[0].get_rename().as_ref().unwrap(),
            "renamed"
        );
        assert_eq!(s.get_selection_list()[0].get_expression(), &expr);
    }

    #[test]
    fn test_selector_select_variable() {
        let var = Variable::new("attr1".to_string());
        let s = Selector::selector().select_variable(var.clone());
        assert_eq!(s.get_selection_list().len(), 1);
        // OutputAttribute::new_from_variable creates an expression from the variable
        // and uses variable name as rename if not specified.
        // Let's assume OutputAttribute has get_rename() and get_expression()
        assert_eq!(
            s.get_selection_list()[0].get_rename().as_ref().unwrap(),
            "attr1"
        );
        match s.get_selection_list()[0].get_expression() {
            Expression::Variable(ref v) => assert_eq!(v, &var),
            _ => panic!("Expected Expression::Variable"),
        }
    }

    #[test]
    fn test_selector_add_selection_list() {
        let oa1 = OutputAttribute::new(
            Some("r1".to_string()),
            Expression::Constant(Constant::int(1)),
        );
        let oa2 = OutputAttribute::new(
            Some("r2".to_string()),
            Expression::Constant(Constant::int(2)),
        );
        let s = Selector::selector().add_selection_list(vec![oa1.clone(), oa2.clone()]);
        assert_eq!(s.get_selection_list().len(), 2);
        assert_eq!(s.get_selection_list()[0], oa1);
        assert_eq!(s.get_selection_list()[1], oa2);
    }

    #[test]
    fn test_selector_group_by() {
        let var = Variable::new("attr_group".to_string());
        let s = Selector::selector().group_by(var.clone());
        assert_eq!(s.get_group_by_list().len(), 1);
        assert_eq!(s.get_group_by_list()[0], var);
    }

    #[test]
    fn test_selector_add_group_by_list() {
        let v1 = Variable::new("g1".to_string());
        let v2 = Variable::new("g2".to_string());
        let s = Selector::selector().add_group_by_list(vec![v1.clone(), v2.clone()]);
        assert_eq!(s.get_group_by_list().len(), 2);
    }

    #[test]
    fn test_selector_having() {
        let expr = Expression::Constant(Constant::bool(true));
        let s = Selector::selector().having(expr.clone());
        assert_eq!(s.get_having_expression().unwrap(), &expr);
    }

    #[test]
    fn test_selector_order_by() {
        let var = Variable::new("attr_order".to_string());
        let s = Selector::selector().order_by(var.clone());
        assert_eq!(s.get_order_by_list().len(), 1);
        assert_eq!(s.get_order_by_list()[0].get_variable(), &var);
        assert_eq!(s.get_order_by_list()[0].get_order(), &OrderByOrder::Asc); // Default order
    }

    #[test]
    fn test_selector_order_by_with_order() {
        let var = Variable::new("attr_order_desc".to_string());
        let s = Selector::selector().order_by_with_order(var.clone(), OrderByOrder::Desc);
        assert_eq!(s.get_order_by_list().len(), 1);
        assert_eq!(s.get_order_by_list()[0].get_variable(), &var);
        assert_eq!(s.get_order_by_list()[0].get_order(), &OrderByOrder::Desc);
    }

    #[test]
    fn test_selector_add_order_by_list() {
        let ob1 = OrderByAttribute::new(Variable::new("o1".to_string()), OrderByOrder::Asc);
        let ob2 = OrderByAttribute::new(Variable::new("o2".to_string()), OrderByOrder::Desc);
        let s = Selector::selector().add_order_by_list(vec![ob1.clone(), ob2.clone()]);
        assert_eq!(s.get_order_by_list().len(), 2);
    }

    #[test]
    fn test_selector_limit() {
        let limit_const_int = Constant::int(10);
        let s_ok = Selector::selector().limit(limit_const_int.clone()).unwrap();
        assert_eq!(s_ok.get_limit().unwrap(), &limit_const_int);

        let limit_const_long = Constant::long(100);
        let s_ok_long = Selector::selector()
            .limit(limit_const_long.clone())
            .unwrap();
        assert_eq!(s_ok_long.get_limit().unwrap(), &limit_const_long);

        let limit_const_str = Constant::string("invalid".to_string());
        let s_err = Selector::selector().limit(limit_const_str);
        assert!(s_err.is_err());
        assert_eq!(
            s_err.unwrap_err(),
            "'limit' only supports int or long constants"
        );
    }

    #[test]
    fn test_selector_offset() {
        let offset_const_int = Constant::int(5);
        let s_ok = Selector::selector()
            .offset(offset_const_int.clone())
            .unwrap();
        assert_eq!(s_ok.get_offset().unwrap(), &offset_const_int);

        let offset_const_long = Constant::long(50);
        let s_ok_long = Selector::selector()
            .offset(offset_const_long.clone())
            .unwrap();
        assert_eq!(s_ok_long.get_offset().unwrap(), &offset_const_long);

        let offset_const_float = Constant::float(5.5);
        let s_err = Selector::selector().offset(offset_const_float);
        assert!(s_err.is_err());
        assert_eq!(
            s_err.unwrap_err(),
            "'offset' only supports int or long constants"
        );
    }
}
