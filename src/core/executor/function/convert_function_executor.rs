use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::query_api::definition::attribute::Type as ApiAttributeType;
use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::util::attribute_converter::get_property_value;
use std::sync::Arc;

#[derive(Debug)]
pub struct ConvertFunctionExecutor {
    value_executor: Box<dyn ExpressionExecutor>,
    target_type: ApiAttributeType,
}

impl ConvertFunctionExecutor {
    pub fn new(
        value_executor: Box<dyn ExpressionExecutor>,
        type_executor: Box<dyn ExpressionExecutor>,
    ) -> Result<Self, String> {
        if type_executor.get_return_type() != ApiAttributeType::STRING {
            return Err("convert() type argument must be STRING".to_string());
        }
        let type_val = match type_executor.execute(None) {
            Some(AttributeValue::String(s)) => s.to_lowercase(),
            _ => return Err("convert() requires constant type string".to_string()),
        };
        let target_type = match type_val.as_str() {
            "string" => ApiAttributeType::STRING,
            "int" => ApiAttributeType::INT,
            "long" => ApiAttributeType::LONG,
            "float" => ApiAttributeType::FLOAT,
            "double" => ApiAttributeType::DOUBLE,
            "bool" | "boolean" => ApiAttributeType::BOOL,
            _ => return Err(format!("Unsupported convert target type: {}", type_val)),
        };
        Ok(Self { value_executor, target_type })
    }
}

impl ExpressionExecutor for ConvertFunctionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let value = self.value_executor.execute(event)?;
        match get_property_value(value, self.target_type.clone()) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }

    fn get_return_type(&self) -> ApiAttributeType {
        self.target_type
    }

    fn clone_executor(&self, ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        Box::new(ConvertFunctionExecutor {
            value_executor: self.value_executor.clone_executor(ctx),
            target_type: self.target_type,
        })
    }
}
