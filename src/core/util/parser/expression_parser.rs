// siddhi_rust/src/core/util/parser/expression_parser.rs

use crate::query_api::{
    expression::{
        Expression as ApiExpression,
        constant::ConstantValueWithFloat as ApiConstantValue
    },
    definition::attribute::Type as ApiAttributeType, // Import Type enum
};
use crate::core::event::complex_event::ComplexEvent; // Added this import
use crate::core::executor::{
    expression_executor::ExpressionExecutor,
    constant_expression_executor::ConstantExpressionExecutor,
    variable_expression_executor::{VariableExpressionExecutor /*, VariablePosition, EventDataArrayType */},
    math::*,
    condition::*,
    function::*,
};
use crate::core::event::value::AttributeValue as CoreAttributeValue;
use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::event::stream::meta_stream_event::MetaStreamEvent;
use crate::core::executor::function::scalar_function_executor::ScalarFunctionExecutor;


use std::sync::Arc;
// use std::collections::HashMap;

// Wrapper executor for calling ScalarFunctionExecutor (UDFs and complex built-ins)
#[derive(Debug)]
pub struct AttributeFunctionExpressionExecutor {
    scalar_function_executor: Box<dyn ScalarFunctionExecutor>,
    argument_executors: Vec<Box<dyn ExpressionExecutor>>,
    // siddhi_app_context: Arc<SiddhiAppContext>, // Stored by scalar_function_executor if needed after init
    return_type: ApiAttributeType,
}

impl AttributeFunctionExpressionExecutor {
    pub fn new(
        mut scalar_func_impl: Box<dyn ScalarFunctionExecutor>,
        arg_execs: Vec<Box<dyn ExpressionExecutor>>,
        app_ctx: Arc<SiddhiAppContext>,
    ) -> Result<Self, String> {
        scalar_func_impl.init(&arg_execs, &app_ctx)?;
        let return_type = scalar_func_impl.get_return_type();
        Ok(Self {
            scalar_function_executor: scalar_func_impl,
            argument_executors: arg_execs,
            return_type,
        })
    }
}

impl ExpressionExecutor for AttributeFunctionExpressionExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<CoreAttributeValue> { // ComplexEvent should now be in scope
        // This simplified model assumes that the ScalarFunctionExecutor's `execute` method,
        // inherited from ExpressionExecutor, will correctly use its initialized state
        // (which might include information about its arguments derived from their executors during `init`)
        // to compute its value based on the incoming event.
        // A more complex model might involve this AttributeFunctionExpressionExecutor first
        // executing its `argument_executors` to get `AttributeValue`s and then passing those
        // values to a different method on `ScalarFunctionExecutor`, e.g., `eval(args: Vec<AttributeValue>)`.
        // For now, we stick to the ExpressionExecutor::execute signature for the UDF.
        self.scalar_function_executor.execute(event)
    }

    fn get_return_type(&self) -> ApiAttributeType {
        self.return_type
    }

    fn clone_executor(&self, siddhi_app_context: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
         let cloned_args = self.argument_executors.iter().map(|e| e.clone_executor(siddhi_app_context)).collect();
         let cloned_scalar_fn = self.scalar_function_executor.clone_scalar_function();
         // Re-initialize the cloned scalar function
         match AttributeFunctionExpressionExecutor::new(cloned_scalar_fn, cloned_args, Arc::clone(siddhi_app_context)) {
            Ok(exec) => Box::new(exec),
            Err(e) => {
                // TODO: Proper error handling. Cloning shouldn't ideally fail if original was valid.
                // This might panic if init logic in a UDF isn't designed to be repeatedly callable on clones,
                // or if context isn't fully available.
                panic!("Failed to clone AttributeFunctionExpressionExecutor due to init error: {}", e);
            }
         }
    }
}


// Simplified context for initial ExpressionParser focusing on single input stream scenarios.
/// Context for ExpressionParser, providing necessary metadata.
/// Current limitations: Assumes a single input stream via `meta_input_event`.
/// Does not yet handle variables from tables, aggregations, window functions, or complex stream joins.
pub struct ExpressionParserContext<'a> { // Added lifetime 'a for query_name
    pub siddhi_app_context: Arc<SiddhiAppContext>,
    pub meta_input_event: Arc<MetaStreamEvent>,
    pub query_name: &'a str, // Added query_name for context in errors/logging
}


/// Parses query_api::Expression into core::ExpressionExecutor instances.
/// Current limitations: Variable resolution is simplified for single input streams.
/// Does not handle full complexity of all expression types or contexts (e.g., aggregations within HAVING).
pub fn parse_expression<'a>( // Added lifetime 'a
    api_expr: &ApiExpression,
    context: &ExpressionParserContext<'a>, // Use context with lifetime
) -> Result<Box<dyn ExpressionExecutor>, String> {
    match api_expr {
        ApiExpression::Constant(api_const) => {
            let (core_value, core_type) = convert_api_constant_to_core_attribute_value(&api_const.value);
            Ok(Box::new(ConstantExpressionExecutor::new(core_value, core_type)))
        }
        ApiExpression::Variable(api_var) => {
            let attribute_name = &api_var.attribute_name;
            if let Some((index, attr_type)) = context.meta_input_event.find_attribute_info(attribute_name) {
                Ok(Box::new(VariableExpressionExecutor::new(*index, attr_type.clone(), attribute_name.to_string())))
            } else {
                Err(format!("Variable '{}' not found in input stream definition provided by MetaStreamEvent for query '{}'.", attribute_name, context.query_name))
            }
        }
        ApiExpression::Add(api_op) => {
            let left_exec = parse_expression(&api_op.left_value, context)?;
            let right_exec = parse_expression(&api_op.right_value, context)?;
            Ok(Box::new(AddExpressionExecutor::new(left_exec, right_exec)?))
        }
        ApiExpression::Subtract(api_op) => {
            let left_exec = parse_expression(&api_op.left_value, context)?;
            let right_exec = parse_expression(&api_op.right_value, context)?;
            Ok(Box::new(SubtractExpressionExecutor::new(left_exec, right_exec)?))
        }
        ApiExpression::Multiply(api_op) => {
            let left_exec = parse_expression(&api_op.left_value, context)?;
            let right_exec = parse_expression(&api_op.right_value, context)?;
            Ok(Box::new(MultiplyExpressionExecutor::new(left_exec, right_exec)?))
        }
        ApiExpression::Divide(api_op) => {
            let left_exec = parse_expression(&api_op.left_value, context)?;
            let right_exec = parse_expression(&api_op.right_value, context)?;
            Ok(Box::new(DivideExpressionExecutor::new(left_exec, right_exec)?))
        }
        ApiExpression::Mod(api_op) => {
            let left_exec = parse_expression(&api_op.left_value, context)?;
            let right_exec = parse_expression(&api_op.right_value, context)?;
            Ok(Box::new(ModExpressionExecutor::new(left_exec, right_exec)?))
        }
        ApiExpression::And(api_op) => {
            let left_exec = parse_expression(&api_op.left_expression, context)?;
            let right_exec = parse_expression(&api_op.right_expression, context)?;
            Ok(Box::new(AndExpressionExecutor::new(left_exec, right_exec)?))
        }
        ApiExpression::Or(api_op) => {
            let left_exec = parse_expression(&api_op.left_expression, context)?;
            let right_exec = parse_expression(&api_op.right_expression, context)?;
            Ok(Box::new(OrExpressionExecutor::new(left_exec, right_exec)?))
        }
        ApiExpression::Not(api_op) => {
            let exec = parse_expression(&api_op.expression, context)?;
            Ok(Box::new(NotExpressionExecutor::new(exec)?))
        }
        ApiExpression::Compare(api_op) => {
            let left_exec = parse_expression(&api_op.left_expression, context)?;
            let right_exec = parse_expression(&api_op.right_expression, context)?;
            Ok(Box::new(CompareExpressionExecutor::new(left_exec, right_exec, api_op.operator.clone())))
        }
        ApiExpression::IsNull(api_op) => {
            if let Some(inner_expr) = &api_op.expression {
                 let exec = parse_expression(inner_expr, context)?;
                 Ok(Box::new(IsNullExpressionExecutor::new(exec)))
            } else {
                Err("IsNull without an inner expression (IsNullStream) not yet fully supported here.".to_string())
            }
        }
        ApiExpression::In(api_op) => {
            let val_exec = parse_expression(&api_op.expression, context)?;
            Ok(Box::new(InExpressionExecutor::new(val_exec, api_op.source_id.clone())))
        }
        ApiExpression::AttributeFunction(api_func) => {
            let mut arg_execs: Vec<Box<dyn ExpressionExecutor>> = Vec::new();
            for arg_expr in &api_func.parameters {
                arg_execs.push(parse_expression(arg_expr, context)?);
            }

            let function_lookup_name = if let Some(ns) = &api_func.extension_namespace {
                if ns.is_empty() { api_func.function_name.clone() }
                else { format!("{}:{}", ns, api_func.function_name) }
            } else {
                api_func.function_name.clone()
            };

            // Try built-in common functions first
            match (api_func.extension_namespace.as_deref(), api_func.function_name.as_str()) {
                (None | Some(""), "coalesce") => Ok(Box::new(CoalesceFunctionExecutor::new(arg_execs)?)),
                (None | Some(""), "ifThenElse") => {
                    if arg_execs.len() == 3 {
                        let else_e = arg_execs.remove(2);
                        let then_e = arg_execs.remove(1);
                        let cond_e = arg_execs.remove(0);
                        Ok(Box::new(IfThenElseFunctionExecutor::new(cond_e, then_e, else_e)?))
                    } else {
                        Err(format!("ifThenElse expects 3 arguments, found {}", arg_execs.len()))
                    }
                }
                (None | Some(""), "uuid") => {
                    if !arg_execs.is_empty() { return Err("uuid() function takes no arguments".to_string()); }
                    Ok(Box::new(UuidFunctionExecutor::new()))
                }
                (None | Some(""), name) if name == "instanceOfBoolean" && arg_execs.len() == 1 => Ok(Box::new(InstanceOfBooleanExpressionExecutor::new(arg_execs.remove(0))?)),
                (None | Some(""), name) if name == "instanceOfString" && arg_execs.len() == 1 => Ok(Box::new(InstanceOfStringExpressionExecutor::new(arg_execs.remove(0))?)),
                (None | Some(""), name) if name == "instanceOfInteger" && arg_execs.len() == 1 => Ok(Box::new(InstanceOfIntegerExpressionExecutor::new(arg_execs.remove(0))?)),
                (None | Some(""), name) if name == "instanceOfLong" && arg_execs.len() == 1 => Ok(Box::new(InstanceOfLongExpressionExecutor::new(arg_execs.remove(0))?)),
                (None | Some(""), name) if name == "instanceOfFloat" && arg_execs.len() == 1 => Ok(Box::new(InstanceOfFloatExpressionExecutor::new(arg_execs.remove(0))?)),
                (None | Some(""), name) if name == "instanceOfDouble" && arg_execs.len() == 1 => Ok(Box::new(InstanceOfDoubleExpressionExecutor::new(arg_execs.remove(0))?)),
                _ => { // UDF lookup from context
                    if let Some(scalar_fn_factory) = context.siddhi_app_context.siddhi_context().get_scalar_function_factory(&function_lookup_name) {
                        Ok(Box::new(AttributeFunctionExpressionExecutor::new(
                            scalar_fn_factory.clone_scalar_function(),
                            arg_execs,
                            Arc::clone(&context.siddhi_app_context)
                        )?))
                    } else {
                        Err(format!("Unsupported or unknown function: {}", function_lookup_name))
                    }
                }
            }
        }
    }
}

fn convert_api_constant_to_core_attribute_value(api_val: &ApiConstantValue) -> (CoreAttributeValue, ApiAttributeType) { // Changed to ApiAttributeType
    match api_val {
        ApiConstantValue::String(s) => (CoreAttributeValue::String(s.clone()), ApiAttributeType::STRING),
        ApiConstantValue::Int(i) => (CoreAttributeValue::Int(*i), ApiAttributeType::INT),
        ApiConstantValue::Long(l) => (CoreAttributeValue::Long(*l), ApiAttributeType::LONG),
        ApiConstantValue::Float(f) => (CoreAttributeValue::Float(*f), ApiAttributeType::FLOAT),
        ApiConstantValue::Double(d) => (CoreAttributeValue::Double(*d), ApiAttributeType::DOUBLE),
        ApiConstantValue::Bool(b) => (CoreAttributeValue::Bool(*b), ApiAttributeType::BOOL),
        ApiConstantValue::Time(t) => (CoreAttributeValue::Long(*t), ApiAttributeType::LONG),
    }
}
