use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::query_api::definition::attribute::Type as ApiAttributeType;
use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::executor::function::*;
use std::sync::Arc;

pub type BuiltinBuilder = fn(Vec<Box<dyn ExpressionExecutor>>) -> Result<Box<dyn ExpressionExecutor>, String>;

pub struct BuiltinScalarFunction {
    pub name: &'static str,
    pub builder: BuiltinBuilder,
    executor: Option<Box<dyn ExpressionExecutor>>,
}

impl Clone for BuiltinScalarFunction {
    fn clone(&self) -> Self {
        Self::new(self.name, self.builder)
    }
}

impl std::fmt::Debug for BuiltinScalarFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuiltinScalarFunction")
            .field("name", &self.name)
            .finish()
    }
}

impl BuiltinScalarFunction {
    pub fn new(name: &'static str, builder: BuiltinBuilder) -> Self {
        Self { name, builder, executor: None }
    }
}

impl ExpressionExecutor for BuiltinScalarFunction {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        self.executor.as_ref()?.execute(event)
    }

    fn get_return_type(&self) -> ApiAttributeType {
        self.executor.as_ref().map(|e| e.get_return_type()).unwrap_or(ApiAttributeType::OBJECT)
    }

    fn clone_executor(&self, ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        let mut cloned = Self::new(self.name, self.builder);
        if let Some(exec) = &self.executor {
            cloned.executor = Some(exec.clone_executor(ctx));
        }
        Box::new(cloned)
    }
}

impl ScalarFunctionExecutor for BuiltinScalarFunction {
    fn init(&mut self, args: &Vec<Box<dyn ExpressionExecutor>>, ctx: &Arc<SiddhiAppContext>) -> Result<(), String> {
        let mut cloned: Vec<Box<dyn ExpressionExecutor>> = args.iter().map(|e| e.clone_executor(ctx)).collect();
        let exec = (self.builder)(cloned)?;
        self.executor = Some(exec);
        Ok(())
    }

    fn destroy(&mut self) {
        self.executor = None;
    }

    fn get_name(&self) -> String {
        self.name.to_string()
    }

    fn clone_scalar_function(&self) -> Box<dyn ScalarFunctionExecutor> {
        Box::new(Self::new(self.name, self.builder))
    }
}

// --- Builtin builder helpers ---

fn build_cast(args: Vec<Box<dyn ExpressionExecutor>>) -> Result<Box<dyn ExpressionExecutor>, String> {
    if args.len() != 2 {
        return Err("cast() requires two arguments".to_string());
    }
    let mut a = args;
    let type_exec = a.remove(1);
    let val_exec = a.remove(0);
    Ok(Box::new(CastFunctionExecutor::new(val_exec, type_exec)?))
}

fn build_convert(args: Vec<Box<dyn ExpressionExecutor>>) -> Result<Box<dyn ExpressionExecutor>, String> {
    if args.len() != 2 {
        return Err("convert() requires two arguments".to_string());
    }
    let mut a = args;
    let type_exec = a.remove(1);
    let val_exec = a.remove(0);
    Ok(Box::new(ConvertFunctionExecutor::new(val_exec, type_exec)?))
}

fn build_concat(args: Vec<Box<dyn ExpressionExecutor>>) -> Result<Box<dyn ExpressionExecutor>, String> {
    Ok(Box::new(ConcatFunctionExecutor::new(args)?))
}

fn build_length(args: Vec<Box<dyn ExpressionExecutor>>) -> Result<Box<dyn ExpressionExecutor>, String> {
    if args.len() != 1 { return Err("length() requires one argument".to_string()); }
    Ok(Box::new(LengthFunctionExecutor::new(args.into_iter().next().unwrap())?))
}

fn build_coalesce(args: Vec<Box<dyn ExpressionExecutor>>) -> Result<Box<dyn ExpressionExecutor>, String> {
    Ok(Box::new(CoalesceFunctionExecutor::new(args)?))
}

fn build_if_then_else(mut args: Vec<Box<dyn ExpressionExecutor>>) -> Result<Box<dyn ExpressionExecutor>, String> {
    if args.len() != 3 { return Err("ifThenElse() requires three arguments".to_string()); }
    let else_e = args.remove(2);
    let then_e = args.remove(1);
    let cond_e = args.remove(0);
    Ok(Box::new(IfThenElseFunctionExecutor::new(cond_e, then_e, else_e)?))
}

fn build_uuid(_args: Vec<Box<dyn ExpressionExecutor>>) -> Result<Box<dyn ExpressionExecutor>, String> {
    Ok(Box::new(UuidFunctionExecutor::new()))
}

fn build_current_timestamp(_args: Vec<Box<dyn ExpressionExecutor>>) -> Result<Box<dyn ExpressionExecutor>, String> {
    Ok(Box::new(CurrentTimestampFunctionExecutor::default()))
}

fn build_format_date(mut args: Vec<Box<dyn ExpressionExecutor>>) -> Result<Box<dyn ExpressionExecutor>, String> {
    if args.len() != 2 { return Err("formatDate() requires two arguments".to_string()); }
    let pattern = args.remove(1);
    let ts = args.remove(0);
    Ok(Box::new(FormatDateFunctionExecutor::new(ts, pattern)?))
}

fn build_round(args: Vec<Box<dyn ExpressionExecutor>>) -> Result<Box<dyn ExpressionExecutor>, String> {
    if args.len() != 1 { return Err("round() requires one argument".to_string()); }
    Ok(Box::new(RoundFunctionExecutor::new(args.into_iter().next().unwrap())?))
}

fn build_sqrt(args: Vec<Box<dyn ExpressionExecutor>>) -> Result<Box<dyn ExpressionExecutor>, String> {
    if args.len() != 1 { return Err("sqrt() requires one argument".to_string()); }
    Ok(Box::new(SqrtFunctionExecutor::new(args.into_iter().next().unwrap())?))
}

fn build_event_timestamp(args: Vec<Box<dyn ExpressionExecutor>>) -> Result<Box<dyn ExpressionExecutor>, String> {
    if args.is_empty() {
        Ok(Box::new(EventTimestampFunctionExecutor::new(None)))
    } else if args.len() == 1 {
        Ok(Box::new(EventTimestampFunctionExecutor::new(Some(args.into_iter().next().unwrap()))))
    } else {
        Err("eventTimestamp() takes zero or one argument".to_string())
    }
}

/// Register default builtin scalar functions into the provided SiddhiContext.
pub fn register_builtin_scalar_functions(ctx: &crate::core::config::siddhi_context::SiddhiContext) {
    ctx.add_scalar_function_factory("cast".to_string(), Box::new(BuiltinScalarFunction::new("cast", build_cast)));
    ctx.add_scalar_function_factory("convert".to_string(), Box::new(BuiltinScalarFunction::new("convert", build_convert)));
    ctx.add_scalar_function_factory("concat".to_string(), Box::new(BuiltinScalarFunction::new("concat", build_concat)));
    ctx.add_scalar_function_factory("str:concat".to_string(), Box::new(BuiltinScalarFunction::new("str:concat", build_concat)));
    ctx.add_scalar_function_factory("length".to_string(), Box::new(BuiltinScalarFunction::new("length", build_length)));
    ctx.add_scalar_function_factory("str:length".to_string(), Box::new(BuiltinScalarFunction::new("str:length", build_length)));
    ctx.add_scalar_function_factory("coalesce".to_string(), Box::new(BuiltinScalarFunction::new("coalesce", build_coalesce)));
    ctx.add_scalar_function_factory("ifThenElse".to_string(), Box::new(BuiltinScalarFunction::new("ifThenElse", build_if_then_else)));
    ctx.add_scalar_function_factory("uuid".to_string(), Box::new(BuiltinScalarFunction::new("uuid", build_uuid)));
    ctx.add_scalar_function_factory("currentTimestamp".to_string(), Box::new(BuiltinScalarFunction::new("currentTimestamp", build_current_timestamp)));
    ctx.add_scalar_function_factory("time:currentTimestamp".to_string(), Box::new(BuiltinScalarFunction::new("time:currentTimestamp", build_current_timestamp)));
    ctx.add_scalar_function_factory("formatDate".to_string(), Box::new(BuiltinScalarFunction::new("formatDate", build_format_date)));
    ctx.add_scalar_function_factory("time:formatDate".to_string(), Box::new(BuiltinScalarFunction::new("time:formatDate", build_format_date)));
    ctx.add_scalar_function_factory("round".to_string(), Box::new(BuiltinScalarFunction::new("round", build_round)));
    ctx.add_scalar_function_factory("math:round".to_string(), Box::new(BuiltinScalarFunction::new("math:round", build_round)));
    ctx.add_scalar_function_factory("sqrt".to_string(), Box::new(BuiltinScalarFunction::new("sqrt", build_sqrt)));
    ctx.add_scalar_function_factory("math:sqrt".to_string(), Box::new(BuiltinScalarFunction::new("math:sqrt", build_sqrt)));
    ctx.add_scalar_function_factory("eventTimestamp".to_string(), Box::new(BuiltinScalarFunction::new("eventTimestamp", build_event_timestamp)));
}
