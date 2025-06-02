// siddhi_rust/src/core/executor/condition/compare_expression_executor.rs
// Corresponds to io.siddhi.core.executor.condition.compare.CompareConditionExpressionExecutor (abstract class)
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::query_api::definition::Attribute;
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
                let result = match self.operator {
                    ConditionCompareOperator::Equal => left == right, // Uses AttributeValue::PartialEq
                    ConditionCompareOperator::NotEqual => left != right, // Uses AttributeValue::PartialEq
                    // Placeholder for other operators (LT, GT, LTE, GTE) which need numeric/string type logic
                    _ => {
                        // log_warn!("Unsupported comparison operator ({:?}) or types for CompareExpressionExecutor", self.operator);
                        return None; // Or Some(AttributeValue::Bool(false)) or error
                    }
                };
                Some(AttributeValue::Bool(result))
            }
            _ => None, // If any side fails to execute, propagate None (or error)
        }
    }

    fn get_return_type(&self) -> Attribute::Type {
        Attribute::Type::BOOL
    }

    // fn clone_executor(&self) -> Box<dyn ExpressionExecutor> {
    //     Box::new(CompareExpressionExecutor {
    //         left_executor: self.left_executor.clone_executor(),
    //         right_executor: self.right_executor.clone_executor(),
    //         operator: self.operator,
    //         // left_type: self.left_type,
    //         // right_type: self.right_type,
    //     })
    // }
}
