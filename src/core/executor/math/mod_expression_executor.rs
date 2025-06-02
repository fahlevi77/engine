// siddhi_rust/src/core/executor/math/mod_expression_executor.rs
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::query_api::definition::Attribute;
use super::common::CoerceNumeric; // Use CoerceNumeric from common.rs

#[derive(Debug)]
pub struct ModExpressionExecutor {
    left_executor: Box<dyn ExpressionExecutor>,
    right_executor: Box<dyn ExpressionExecutor>,
    return_type: Attribute::Type, // Modulo result type usually matches operand types after promotion
}

impl ModExpressionExecutor {
    pub fn new(left: Box<dyn ExpressionExecutor>, right: Box<dyn ExpressionExecutor>) -> Result<Self, String> {
        let left_type = left.get_return_type();
        let right_type = right.get_return_type();

        // Type promotion for modulo: result type is usually the type of the dividend after promoting both to a common numeric type.
        // Siddhi's typed ModExpressionExecutors (Int, Long, Float, Double) imply the result type matches this promoted type.
        let return_type = match (left_type, right_type) {
            (Attribute::Type::STRING, _) | (_, Attribute::Type::STRING) |
            (Attribute::Type::BOOL, _) | (_, Attribute::Type::BOOL) |
            (Attribute::Type::OBJECT, _) | (_, Attribute::Type::OBJECT) => {
                return Err(format!("Modulo not supported for input types {:?} and {:?}", left_type, right_type));
            }
            (Attribute::Type::DOUBLE, _) | (_, Attribute::Type::DOUBLE) => Attribute::Type::DOUBLE,
            (Attribute::Type::FLOAT, _) | (_, Attribute::Type::FLOAT) => Attribute::Type::FLOAT,
            (Attribute::Type::LONG, _) | (_, Attribute::Type::LONG) => Attribute::Type::LONG,
            (Attribute::Type::INT, _) | (_, Attribute::Type::INT) => Attribute::Type::INT, // Base case
            _ => return Err(format!("Modulo not supported for incompatible types: {:?} and {:?}", left_type, right_type)),
        };
        Ok(Self { left_executor: left, right_executor: right, return_type })
    }
}

impl ExpressionExecutor for ModExpressionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let left_val_opt = self.left_executor.execute(event);
        let right_val_opt = self.right_executor.execute(event);

        match (left_val_opt, right_val_opt) {
            (Some(left_val), Some(right_val)) => {
                if matches!(left_val, AttributeValue::Null) || matches!(right_val, AttributeValue::Null) {
                    return Some(AttributeValue::Null);
                }

                // Division by zero check for modulo
                let temp_r_for_zero_check = right_val.to_f64_or_err_str("Mod")?; // Check as f64 to catch all numeric zeros
                if temp_r_for_zero_check == 0.0 {
                    // log_error!("Modulo by zero error");
                    return Some(AttributeValue::Null);
                }

                match self.return_type {
                    Attribute::Type::INT => {
                        let l = left_val.to_i32_or_err_str("Mod")?;
                        let r = right_val.to_i32_or_err_str("Mod")?;
                        if r == 0 { return Some(AttributeValue::Null); } // Explicit check for integer 0
                        Some(AttributeValue::Int(l % r))
                    }
                    Attribute::Type::LONG => {
                        let l = left_val.to_i64_or_err_str("Mod")?;
                        let r = right_val.to_i64_or_err_str("Mod")?;
                        if r == 0 { return Some(AttributeValue::Null); }
                        Some(AttributeValue::Long(l % r))
                    }
                    Attribute::Type::FLOAT => {
                        let l = left_val.to_f32_or_err_str("Mod")?;
                        let r = right_val.to_f32_or_err_str("Mod")?;
                        if r == 0.0 { return Some(AttributeValue::Null); }
                        Some(AttributeValue::Float(l % r))
                    }
                    Attribute::Type::DOUBLE => {
                        let l = left_val.to_f64_or_err_str("Mod")?;
                        let r = right_val.to_f64_or_err_str("Mod")?;
                        if r == 0.0 { return Some(AttributeValue::Null); }
                        Some(AttributeValue::Double(l % r))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
    fn get_return_type(&self) -> Attribute::Type { self.return_type }
    // fn clone_executor(&self) -> Box<dyn ExpressionExecutor> { /* ... */ }
}
