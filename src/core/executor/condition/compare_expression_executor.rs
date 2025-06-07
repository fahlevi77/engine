// siddhi_rust/src/core/executor/condition/compare_expression_executor.rs
// Corresponds to io.siddhi.core.executor.condition.compare.CompareConditionExpressionExecutor (abstract class)
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::query_api::definition::attribute::Type as ApiAttributeType; // Import Type enum
// Use ConditionCompareOperator from query_api, as it's part of the expression definition
use crate::query_api::expression::condition::CompareOperator as ConditionCompareOperator;


#[derive(Debug)]
pub struct CompareExpressionExecutor {
    left_executor: Box<dyn ExpressionExecutor>,
    right_executor: Box<dyn ExpressionExecutor>,
    operator: ConditionCompareOperator,
    // Store return types for optimized comparison if known at construction
    // left_type: Attribute::Type,
    // right_type: Attribute::Type,
}

impl CompareExpressionExecutor {
   pub fn new(left: Box<dyn ExpressionExecutor>, right: Box<dyn ExpressionExecutor>, op: ConditionCompareOperator) -> Self {
       // let left_type = left.get_return_type();
       // let right_type = right.get_return_type();
       // TODO: Could perform type compatibility checks here if needed, or select a typed comparison strategy.
       Self {
           left_executor: left,
           right_executor: right,
           operator: op,
           // left_type,
           // right_type,
       }
   }
}

impl ExpressionExecutor for CompareExpressionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let left_opt = self.left_executor.execute(event);
        let right_opt = self.right_executor.execute(event);

        match (left_opt, right_opt) {
            (Some(left), Some(right)) => {
                // Java CompareConditionExpressionExecutor: return !(left == null || right == null) && execute(left, right);
                // This means if either is AttributeValue::Null, the comparison is effectively false for the condition.
                // (Except for IS NULL checks, which are handled by IsNullExpressionExecutor).
                if matches!(left, AttributeValue::Null) || matches!(right, AttributeValue::Null) {
                    // For most operators, if one operand is Null, the result is false or Null.
                    // For IS NULL / IS NOT NULL, it's different.
                    // Let's assume standard SQL behavior: most comparisons with NULL yield NULL (effectively false in boolean context).
                    // For strictness, if we want to return AttributeValue::Null for comparisons involving null:
                    // return Some(AttributeValue::Null);
                    // For now, returning false as per Java's `!(left == null || right == null)` implies this path leads to overall false.
                    return Some(AttributeValue::Bool(false));
                }
                // TODO: Implement detailed comparison logic for all AttributeValue types and operators.
                // This is complex: type coercion (e.g. Int vs Long vs Float vs Double), string compares etc.
                // This requires a robust type promotion and comparison system.
                // Example simplified comparison (primarily for equality, and only if types are somewhat aligned):
                // Updated comparison logic
                let result = match (&left, &right) {
                    // Integer comparisons
                    (AttributeValue::Int(l), AttributeValue::Int(r)) => match self.operator {
                        ConditionCompareOperator::Equal => l == r,
                        ConditionCompareOperator::NotEqual => l != r,
                        ConditionCompareOperator::GreaterThan => l > r,
                        ConditionCompareOperator::GreaterThanEqual => l >= r,
                        ConditionCompareOperator::LessThan => l < r,
                        ConditionCompareOperator::LessThanEqual => l <= r,
                    },
                    // Long comparisons
                    (AttributeValue::Long(l), AttributeValue::Long(r)) => match self.operator {
                        ConditionCompareOperator::Equal => l == r,
                        ConditionCompareOperator::NotEqual => l != r,
                        ConditionCompareOperator::GreaterThan => l > r,
                        ConditionCompareOperator::GreaterThanEqual => l >= r,
                        ConditionCompareOperator::LessThan => l < r,
                        ConditionCompareOperator::LessThanEqual => l <= r,
                    },
                    // Float comparisons
                    (AttributeValue::Float(l), AttributeValue::Float(r)) => match self.operator {
                        ConditionCompareOperator::Equal => (l - r).abs() < f32::EPSILON, // Approx equal for floats
                        ConditionCompareOperator::NotEqual => (l - r).abs() >= f32::EPSILON,
                        ConditionCompareOperator::GreaterThan => l > r,
                        ConditionCompareOperator::GreaterThanEqual => l >= r,
                        ConditionCompareOperator::LessThan => l < r,
                        ConditionCompareOperator::LessThanEqual => l <= r,
                    },
                    // Double comparisons
                    (AttributeValue::Double(l), AttributeValue::Double(r)) => match self.operator {
                        ConditionCompareOperator::Equal => (l - r).abs() < f64::EPSILON, // Approx equal for doubles
                        ConditionCompareOperator::NotEqual => (l - r).abs() >= f64::EPSILON,
                        ConditionCompareOperator::GreaterThan => l > r,
                        ConditionCompareOperator::GreaterThanEqual => l >= r,
                        ConditionCompareOperator::LessThan => l < r,
                        ConditionCompareOperator::LessThanEqual => l <= r,
                    },
                    // String comparisons with ordering
                    (AttributeValue::String(l), AttributeValue::String(r)) => {
                        match self.operator {
                            ConditionCompareOperator::Equal => l == r,
                            ConditionCompareOperator::NotEqual => l != r,
                            ConditionCompareOperator::GreaterThan => l > r,
                            ConditionCompareOperator::GreaterThanEqual => l >= r,
                            ConditionCompareOperator::LessThan => l < r,
                            ConditionCompareOperator::LessThanEqual => l <= r,
                        }
                    }
                    // Bool comparisons (false < true)
                    (AttributeValue::Bool(l), AttributeValue::Bool(r)) => {
                        let l_val = if *l { 1 } else { 0 };
                        let r_val = if *r { 1 } else { 0 };
                        match self.operator {
                            ConditionCompareOperator::Equal => l_val == r_val,
                            ConditionCompareOperator::NotEqual => l_val != r_val,
                            ConditionCompareOperator::GreaterThan => l_val > r_val,
                            ConditionCompareOperator::GreaterThanEqual => l_val >= r_val,
                            ConditionCompareOperator::LessThan => l_val < r_val,
                            ConditionCompareOperator::LessThanEqual => l_val <= r_val,
                        }
                    }
                    // Mixed numeric comparisons with coercion
                    (AttributeValue::Int(l), AttributeValue::Long(r)) => {
                        let l = *l as i64;
                        let r = *r;
                        match self.operator {
                            ConditionCompareOperator::Equal => l == r,
                            ConditionCompareOperator::NotEqual => l != r,
                            ConditionCompareOperator::GreaterThan => l > r,
                            ConditionCompareOperator::GreaterThanEqual => l >= r,
                            ConditionCompareOperator::LessThan => l < r,
                            ConditionCompareOperator::LessThanEqual => l <= r,
                        }
                    }
                    (AttributeValue::Long(l), AttributeValue::Int(r)) => {
                        let l = *l;
                        let r = *r as i64;
                        match self.operator {
                            ConditionCompareOperator::Equal => l == r,
                            ConditionCompareOperator::NotEqual => l != r,
                            ConditionCompareOperator::GreaterThan => l > r,
                            ConditionCompareOperator::GreaterThanEqual => l >= r,
                            ConditionCompareOperator::LessThan => l < r,
                            ConditionCompareOperator::LessThanEqual => l <= r,
                        }
                    }
                    (AttributeValue::Int(l), AttributeValue::Float(r)) => {
                        let l = *l as f32;
                        let r = *r;
                        match self.operator {
                            ConditionCompareOperator::Equal => (l - r).abs() < f32::EPSILON,
                            ConditionCompareOperator::NotEqual => (l - r).abs() >= f32::EPSILON,
                            ConditionCompareOperator::GreaterThan => l > r,
                            ConditionCompareOperator::GreaterThanEqual => l >= r,
                            ConditionCompareOperator::LessThan => l < r,
                            ConditionCompareOperator::LessThanEqual => l <= r,
                        }
                    }
                    (AttributeValue::Float(l), AttributeValue::Int(r)) => {
                        let l = *l;
                        let r = *r as f32;
                        match self.operator {
                            ConditionCompareOperator::Equal => (l - r).abs() < f32::EPSILON,
                            ConditionCompareOperator::NotEqual => (l - r).abs() >= f32::EPSILON,
                            ConditionCompareOperator::GreaterThan => l > r,
                            ConditionCompareOperator::GreaterThanEqual => l >= r,
                            ConditionCompareOperator::LessThan => l < r,
                            ConditionCompareOperator::LessThanEqual => l <= r,
                        }
                    }
                    (AttributeValue::Int(l), AttributeValue::Double(r)) => {
                        let l = *l as f64;
                        let r = *r;
                        match self.operator {
                            ConditionCompareOperator::Equal => (l - r).abs() < f64::EPSILON,
                            ConditionCompareOperator::NotEqual => (l - r).abs() >= f64::EPSILON,
                            ConditionCompareOperator::GreaterThan => l > r,
                            ConditionCompareOperator::GreaterThanEqual => l >= r,
                            ConditionCompareOperator::LessThan => l < r,
                            ConditionCompareOperator::LessThanEqual => l <= r,
                        }
                    }
                    (AttributeValue::Double(l), AttributeValue::Int(r)) => {
                        let l = *l;
                        let r = *r as f64;
                        match self.operator {
                            ConditionCompareOperator::Equal => (l - r).abs() < f64::EPSILON,
                            ConditionCompareOperator::NotEqual => (l - r).abs() >= f64::EPSILON,
                            ConditionCompareOperator::GreaterThan => l > r,
                            ConditionCompareOperator::GreaterThanEqual => l >= r,
                            ConditionCompareOperator::LessThan => l < r,
                            ConditionCompareOperator::LessThanEqual => l <= r,
                        }
                    }
                    (AttributeValue::Long(l), AttributeValue::Float(r)) => {
                        let l = *l as f32;
                        let r = *r;
                        match self.operator {
                            ConditionCompareOperator::Equal => (l - r).abs() < f32::EPSILON,
                            ConditionCompareOperator::NotEqual => (l - r).abs() >= f32::EPSILON,
                            ConditionCompareOperator::GreaterThan => l > r,
                            ConditionCompareOperator::GreaterThanEqual => l >= r,
                            ConditionCompareOperator::LessThan => l < r,
                            ConditionCompareOperator::LessThanEqual => l <= r,
                        }
                    }
                    (AttributeValue::Float(l), AttributeValue::Long(r)) => {
                        let l = *l;
                        let r = *r as f32;
                        match self.operator {
                            ConditionCompareOperator::Equal => (l - r).abs() < f32::EPSILON,
                            ConditionCompareOperator::NotEqual => (l - r).abs() >= f32::EPSILON,
                            ConditionCompareOperator::GreaterThan => l > r,
                            ConditionCompareOperator::GreaterThanEqual => l >= r,
                            ConditionCompareOperator::LessThan => l < r,
                            ConditionCompareOperator::LessThanEqual => l <= r,
                        }
                    }
                    (AttributeValue::Long(l), AttributeValue::Double(r)) => {
                        let l = *l as f64;
                        let r = *r;
                        match self.operator {
                            ConditionCompareOperator::Equal => (l - r).abs() < f64::EPSILON,
                            ConditionCompareOperator::NotEqual => (l - r).abs() >= f64::EPSILON,
                            ConditionCompareOperator::GreaterThan => l > r,
                            ConditionCompareOperator::GreaterThanEqual => l >= r,
                            ConditionCompareOperator::LessThan => l < r,
                            ConditionCompareOperator::LessThanEqual => l <= r,
                        }
                    }
                    (AttributeValue::Double(l), AttributeValue::Long(r)) => {
                        let l = *l;
                        let r = *r as f64;
                        match self.operator {
                            ConditionCompareOperator::Equal => (l - r).abs() < f64::EPSILON,
                            ConditionCompareOperator::NotEqual => (l - r).abs() >= f64::EPSILON,
                            ConditionCompareOperator::GreaterThan => l > r,
                            ConditionCompareOperator::GreaterThanEqual => l >= r,
                            ConditionCompareOperator::LessThan => l < r,
                            ConditionCompareOperator::LessThanEqual => l <= r,
                        }
                    }
                    (AttributeValue::Float(l), AttributeValue::Double(r)) => {
                        let l = *l as f64;
                        let r = *r;
                        match self.operator {
                            ConditionCompareOperator::Equal => (l - r).abs() < f64::EPSILON,
                            ConditionCompareOperator::NotEqual => (l - r).abs() >= f64::EPSILON,
                            ConditionCompareOperator::GreaterThan => l > r,
                            ConditionCompareOperator::GreaterThanEqual => l >= r,
                            ConditionCompareOperator::LessThan => l < r,
                            ConditionCompareOperator::LessThanEqual => l <= r,
                        }
                    }
                    (AttributeValue::Double(l), AttributeValue::Float(r)) => {
                        let l = *l;
                        let r = *r as f64;
                        match self.operator {
                            ConditionCompareOperator::Equal => (l - r).abs() < f64::EPSILON,
                            ConditionCompareOperator::NotEqual => (l - r).abs() >= f64::EPSILON,
                            ConditionCompareOperator::GreaterThan => l > r,
                            ConditionCompareOperator::GreaterThanEqual => l >= r,
                            ConditionCompareOperator::LessThan => l < r,
                            ConditionCompareOperator::LessThanEqual => l <= r,
                        }
                    }
                    _ => {
                        return Some(AttributeValue::Bool(false));
                    }
                };
                Some(AttributeValue::Bool(result))
            }
            _ => None, // If any side fails to execute, propagate None (or error)
        }
    }

    fn get_return_type(&self) -> ApiAttributeType {
        ApiAttributeType::BOOL
    }

    fn clone_executor(&self, siddhi_app_context: &std::sync::Arc<crate::core::config::siddhi_app_context::SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        Box::new(CompareExpressionExecutor {
            left_executor: self.left_executor.clone_executor(siddhi_app_context),
            right_executor: self.right_executor.clone_executor(siddhi_app_context),
            operator: self.operator,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query_api::expression::condition::compare::Operator as ApiCompareOperator;
    use crate::core::executor::constant_expression_executor::ConstantExpressionExecutor;
    use crate::core::event::value::AttributeValue;
    // ApiAttributeType is imported in the outer scope
    use crate::core::executor::expression_executor::ExpressionExecutor;
    use std::sync::Arc;
    use crate::core::config::siddhi_app_context::SiddhiAppContext;

    #[test]
    fn test_compare_greater_than_int_true() { // Renamed as per subtask suggestion (though original would be fine too)
        let left_exec = Box::new(ConstantExpressionExecutor::new(AttributeValue::Int(20), ApiAttributeType::INT));
        let right_exec = Box::new(ConstantExpressionExecutor::new(AttributeValue::Int(10), ApiAttributeType::INT));
        let cmp_exec = CompareExpressionExecutor::new(left_exec, right_exec, ApiCompareOperator::GreaterThan);

        let result = cmp_exec.execute(None);
        assert_eq!(result, Some(AttributeValue::Bool(true)));
        assert_eq!(cmp_exec.get_return_type(), ApiAttributeType::BOOL);
    }

    #[test]
    fn test_compare_less_than_int_false() {
        let left_exec = Box::new(ConstantExpressionExecutor::new(AttributeValue::Int(20), ApiAttributeType::INT));
        let right_exec = Box::new(ConstantExpressionExecutor::new(AttributeValue::Int(10), ApiAttributeType::INT));
        let cmp_exec = CompareExpressionExecutor::new(left_exec, right_exec, ApiCompareOperator::LessThan);

        let result = cmp_exec.execute(None);
        assert_eq!(result, Some(AttributeValue::Bool(false)));
    }

    #[test]
    fn test_compare_equal_float_true() {
        let left_exec = Box::new(ConstantExpressionExecutor::new(AttributeValue::Float(10.5), ApiAttributeType::FLOAT));
        let right_exec = Box::new(ConstantExpressionExecutor::new(AttributeValue::Float(10.5), ApiAttributeType::FLOAT));
        let cmp_exec = CompareExpressionExecutor::new(left_exec, right_exec, ApiCompareOperator::Equal);

        let result = cmp_exec.execute(None);
        assert_eq!(result, Some(AttributeValue::Bool(true)));
    }

    #[test]
    fn test_compare_not_equal_string_true() {
        let left_exec = Box::new(ConstantExpressionExecutor::new(AttributeValue::String("hello".to_string()), ApiAttributeType::STRING));
        let right_exec = Box::new(ConstantExpressionExecutor::new(AttributeValue::String("world".to_string()), ApiAttributeType::STRING));
        let cmp_exec = CompareExpressionExecutor::new(left_exec, right_exec, ApiCompareOperator::NotEqual);

        let result = cmp_exec.execute(None);
        assert_eq!(result, Some(AttributeValue::Bool(true)));
    }

    #[test]
    fn test_compare_with_null_operand() {
        let left_exec = Box::new(ConstantExpressionExecutor::new(AttributeValue::Int(20), ApiAttributeType::INT));
        let right_exec = Box::new(ConstantExpressionExecutor::new(AttributeValue::Null, ApiAttributeType::INT)); // Null operand
        let cmp_exec = CompareExpressionExecutor::new(left_exec, right_exec, ApiCompareOperator::GreaterThan);

        // Current logic: if either operand is Null, returns Bool(false)
        let result = cmp_exec.execute(None);
        assert_eq!(result, Some(AttributeValue::Bool(false)));
    }

    #[test]
    fn test_compare_incompatible_types() {
        let left_exec = Box::new(ConstantExpressionExecutor::new(AttributeValue::Int(20), ApiAttributeType::INT));
        let right_exec = Box::new(ConstantExpressionExecutor::new(AttributeValue::String("text".to_string()), ApiAttributeType::STRING));
        let cmp_exec = CompareExpressionExecutor::new(left_exec, right_exec, ApiCompareOperator::GreaterThan);

        // Current logic: if types are incompatible for the operation, returns Bool(false)
        let result = cmp_exec.execute(None);
        assert_eq!(result, Some(AttributeValue::Bool(false)));
    }

    #[test]
    fn test_compare_clone() {
        let left_exec = Box::new(ConstantExpressionExecutor::new(AttributeValue::Long(100), ApiAttributeType::LONG));
        let right_exec = Box::new(ConstantExpressionExecutor::new(AttributeValue::Long(50), ApiAttributeType::LONG));
        let cmp_exec = CompareExpressionExecutor::new(left_exec, right_exec, ApiCompareOperator::GreaterThanEqual);

        let app_ctx_placeholder = Arc::new(SiddhiAppContext::default_for_testing());
        let cloned_exec = cmp_exec.clone_executor(&app_ctx_placeholder);

        let result = cloned_exec.execute(None);
        assert_eq!(result, Some(AttributeValue::Bool(true)));
        assert_eq!(cloned_exec.get_return_type(), ApiAttributeType::BOOL);
    }

    #[test]
    fn test_operator_equal_int() {
        let left = Box::new(ConstantExpressionExecutor::new(AttributeValue::Int(5), ApiAttributeType::INT));
        let right = Box::new(ConstantExpressionExecutor::new(AttributeValue::Int(5), ApiAttributeType::INT));
        let cmp = CompareExpressionExecutor::new(left, right, ApiCompareOperator::Equal);
        assert_eq!(cmp.execute(None), Some(AttributeValue::Bool(true)));
    }

    #[test]
    fn test_operator_not_equal_long() {
        let left = Box::new(ConstantExpressionExecutor::new(AttributeValue::Long(5), ApiAttributeType::LONG));
        let right = Box::new(ConstantExpressionExecutor::new(AttributeValue::Long(10), ApiAttributeType::LONG));
        let cmp = CompareExpressionExecutor::new(left, right, ApiCompareOperator::NotEqual);
        assert_eq!(cmp.execute(None), Some(AttributeValue::Bool(true)));
    }

    #[test]
    fn test_operator_greater_than_cross() {
        let left = Box::new(ConstantExpressionExecutor::new(AttributeValue::Float(7.5), ApiAttributeType::FLOAT));
        let right = Box::new(ConstantExpressionExecutor::new(AttributeValue::Int(7), ApiAttributeType::INT));
        let cmp = CompareExpressionExecutor::new(left, right, ApiCompareOperator::GreaterThan);
        assert_eq!(cmp.execute(None), Some(AttributeValue::Bool(true)));
    }

    #[test]
    fn test_operator_greater_than_equal_cross() {
        let left = Box::new(ConstantExpressionExecutor::new(AttributeValue::Double(7.0), ApiAttributeType::DOUBLE));
        let right = Box::new(ConstantExpressionExecutor::new(AttributeValue::Long(7), ApiAttributeType::LONG));
        let cmp = CompareExpressionExecutor::new(left, right, ApiCompareOperator::GreaterThanEqual);
        assert_eq!(cmp.execute(None), Some(AttributeValue::Bool(true)));
    }

    #[test]
    fn test_operator_less_than_string() {
        let left = Box::new(ConstantExpressionExecutor::new(AttributeValue::String("apple".to_string()), ApiAttributeType::STRING));
        let right = Box::new(ConstantExpressionExecutor::new(AttributeValue::String("banana".to_string()), ApiAttributeType::STRING));
        let cmp = CompareExpressionExecutor::new(left, right, ApiCompareOperator::LessThan);
        assert_eq!(cmp.execute(None), Some(AttributeValue::Bool(true)));
    }

    #[test]
    fn test_operator_less_than_equal_bool() {
        let left = Box::new(ConstantExpressionExecutor::new(AttributeValue::Bool(false), ApiAttributeType::BOOL));
        let right = Box::new(ConstantExpressionExecutor::new(AttributeValue::Bool(true), ApiAttributeType::BOOL));
        let cmp = CompareExpressionExecutor::new(left, right, ApiCompareOperator::LessThanEqual);
        assert_eq!(cmp.execute(None), Some(AttributeValue::Bool(true)));
    }
}
