// siddhi_rust/src/core/executor/math/multiply.rs
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::query_api::definition::Attribute;
use super::common::CoerceNumeric; // Use CoerceNumeric from common.rs

#[derive(Debug)]
pub struct MultiplyExpressionExecutor {
    left_executor: Box<dyn ExpressionExecutor>,
    right_executor: Box<dyn ExpressionExecutor>,
    return_type: Attribute::Type,
}

impl MultiplyExpressionExecutor {
    pub fn new(left: Box<dyn ExpressionExecutor>, right: Box<dyn ExpressionExecutor>) -> Result<Self, String> {
        let left_type = left.get_return_type();
        let right_type = right.get_return_type();

        let return_type = match (left_type, right_type) {
            (Attribute::Type::STRING, _) | (_, Attribute::Type::STRING) |
            (Attribute::Type::BOOL, _) | (_, Attribute::Type::BOOL) |
            (Attribute::Type::OBJECT, _) | (_, Attribute::Type::OBJECT) => {
                return Err(format!("Multiplication not supported for input types {:?} and {:?}", left_type, right_type));
            }
            (Attribute::Type::DOUBLE, _) | (_, Attribute::Type::DOUBLE) => Attribute::Type::DOUBLE,
            (Attribute::Type::FLOAT, _) | (_, Attribute::Type::FLOAT) => Attribute::Type::FLOAT,
            (Attribute::Type::LONG, _) | (_, Attribute::Type::LONG) => Attribute::Type::LONG,
            (Attribute::Type::INT, _) | (_, Attribute::Type::INT) => Attribute::Type::INT,
             _ => return Err(format!("Multiplication not supported for incompatible types: {:?} and {:?}", left_type, right_type)),
        };
        Ok(Self { left_executor: left, right_executor: right, return_type })
    }
}

impl ExpressionExecutor for MultiplyExpressionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let left_val_opt = self.left_executor.execute(event);
        let right_val_opt = self.right_executor.execute(event);

        match (left_val_opt, right_val_opt) {
            (Some(left_val), Some(right_val)) => {
                if matches!(left_val, AttributeValue::Null) || matches!(right_val, AttributeValue::Null) {
                    return Some(AttributeValue::Null);
                }
                match self.return_type {
                    Attribute::Type::INT => {
                        let l = left_val.to_i32_or_err_str("Multiply")?;
                        let r = right_val.to_i32_or_err_str("Multiply")?;
                        Some(AttributeValue::Int(l.wrapping_mul(r)))
                    }
                    Attribute::Type::LONG => {
                        let l = left_val.to_i64_or_err_str("Multiply")?;
                        let r = right_val.to_i64_or_err_str("Multiply")?;
                        Some(AttributeValue::Long(l.wrapping_mul(r)))
                    }
                    Attribute::Type::FLOAT => {
                        let l = left_val.to_f32_or_err_str("Multiply")?;
                        let r = right_val.to_f32_or_err_str("Multiply")?;
                        Some(AttributeValue::Float(l * r))
                    }
                    Attribute::Type::DOUBLE => {
                        let l = left_val.to_f64_or_err_str("Multiply")?;
                        let r = right_val.to_f64_or_err_str("Multiply")?;
                        Some(AttributeValue::Double(l * r))
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
