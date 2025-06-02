// siddhi_rust/src/core/executor/math/divide.rs
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::query_api::definition::Attribute;
use super::common::CoerceNumeric; // Use CoerceNumeric from common.rs

#[derive(Debug)]
pub struct DivideExpressionExecutor {
    left_executor: Box<dyn ExpressionExecutor>,
    right_executor: Box<dyn ExpressionExecutor>,
    return_type: Attribute::Type, // Division often results in float/double
}

impl DivideExpressionExecutor {
    pub fn new(left: Box<dyn ExpressionExecutor>, right: Box<dyn ExpressionExecutor>) -> Result<Self, String> {
        let left_type = left.get_return_type();
        let right_type = right.get_return_type();

        // Determine return type: typically Double for division, unless both are Int/Long and result should be Int/Long (integer division).
        // Siddhi's Java typed executors (e.g., DivideExpressionExecutorInt) suggest it maintains integer division if possible.
        // However, to avoid loss of precision and match general SQL behavior for `/`, promoting to DOUBLE is safest.
        // If strict integer division is needed, a separate DivInt executor or different operator could be used.
        // For now, promoting to DOUBLE for `/` operator.
        let return_type = match (left_type, right_type) {
             (Attribute::Type::STRING, _) | (_, Attribute::Type::STRING) |
            (Attribute::Type::BOOL, _) | (_, Attribute::Type::BOOL) |
            (Attribute::Type::OBJECT, _) | (_, Attribute::Type::OBJECT) => {
                return Err(format!("Division not supported for input types {:?} and {:?}", left_type, right_type));
            }
            // Any division involving numbers results in DOUBLE for safety and precision by default
            _ => Attribute::Type::DOUBLE,
        };
        // One could refine this: if both are INT, maybe return INT (integer division), or if both are LONG, return LONG.
        // Or, if strict type matching like Java's per-type executors is desired:
        // let return_type = match (left_type, right_type) {
        //     (Attribute::Type::DOUBLE, _) | (_, Attribute::Type::DOUBLE) => Attribute::Type::DOUBLE,
        //     (Attribute::Type::FLOAT, _) | (_, Attribute::Type::FLOAT) => Attribute::Type::FLOAT,
        //     (Attribute::Type::LONG, _) | (_, Attribute::Type::LONG) => Attribute::Type::LONG, // if long div long -> long
        //     (Attribute::Type::INT, _) | (_, Attribute::Type::INT) => Attribute::Type::INT, // if int div int -> int
        //     _ => return Err(...)
        // };
        // The prompt example for Add determined return type by promoting. Let's follow that.
        // The Add example promotes to the "largest" type present. Division is trickier.
        // For now, will default to Double as per the prompt's general approach of a single executor per op.

        Ok(Self { left_executor: left, right_executor: right, return_type })
    }
}

impl ExpressionExecutor for DivideExpressionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let left_val_opt = self.left_executor.execute(event);
        let right_val_opt = self.right_executor.execute(event);

        match (left_val_opt, right_val_opt) {
            (Some(left_val), Some(right_val)) => {
                if matches!(left_val, AttributeValue::Null) || matches!(right_val, AttributeValue::Null) {
                    return Some(AttributeValue::Null);
                }

                // Coerce both to f64 for division, then cast to return_type if needed (though return_type is usually DOUBLE)
                let l = left_val.to_f64_or_err_str("Divide")?;
                let r = right_val.to_f64_or_err_str("Divide")?;

                if r == 0.0 {
                    // Division by zero. Siddhi might return null or throw an error.
                    // Returning Null is a common SQL behavior.
                    // log_error!("Division by zero error");
                    return Some(AttributeValue::Null);
                }
                let result = l / r;

                match self.return_type { // This logic might be simplified if return_type is always DOUBLE
                    Attribute::Type::DOUBLE => Some(AttributeValue::Double(result)),
                    Attribute::Type::FLOAT => Some(AttributeValue::Float(result as f32)),
                    Attribute::Type::LONG => Some(AttributeValue::Long(result as i64)), // Truncating
                    Attribute::Type::INT => Some(AttributeValue::Int(result as i32)), // Truncating
                    _ => {
                        // log_error!("Division resulted in unexpected return type: {:?}", self.return_type);
                        None
                    }
                }
            }
            _ => None,
        }
    }
    fn get_return_type(&self) -> Attribute::Type { self.return_type }
    // fn clone_executor(&self) -> Box<dyn ExpressionExecutor> { /* ... */ }
}
