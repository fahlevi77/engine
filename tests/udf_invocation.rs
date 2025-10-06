// TODO: NOT PART OF M1 - UDF invocation test uses old SiddhiQL syntax
// Test uses "define stream" which is not supported by SQL parser.
// See feat/grammar/GRAMMAR_STATUS.md for M1 feature list.

#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::core::config::siddhi_app_context::SiddhiAppContext;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::executor::expression_executor::ExpressionExecutor;
use siddhi_rust::core::executor::function::scalar_function_executor::ScalarFunctionExecutor;
use siddhi_rust::core::siddhi_manager::SiddhiManager;
use siddhi_rust::query_api::definition::attribute::Type as AttrType;
use siddhi_rust::query_api::siddhi_app::SiddhiApp;
use std::sync::Arc;

#[derive(Debug, Default)]
struct PlusOneFn {
    arg: Option<Box<dyn ExpressionExecutor>>,
}

impl Clone for PlusOneFn {
    fn clone(&self) -> Self {
        Self { arg: None }
    }
}

impl ExpressionExecutor for PlusOneFn {
    fn execute(
        &self,
        event: Option<&dyn siddhi_rust::core::event::complex_event::ComplexEvent>,
    ) -> Option<AttributeValue> {
        let v = self.arg.as_ref()?.execute(event)?;
        match v {
            AttributeValue::Int(i) => Some(AttributeValue::Int(i + 1)),
            _ => None,
        }
    }

    fn get_return_type(&self) -> AttrType {
        AttrType::INT
    }

    fn clone_executor(&self, _ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        Box::new(self.clone())
    }
}

impl ScalarFunctionExecutor for PlusOneFn {
    fn init(
        &mut self,
        args: &Vec<Box<dyn ExpressionExecutor>>,
        ctx: &Arc<SiddhiAppContext>,
    ) -> Result<(), String> {
        if args.len() != 1 {
            return Err("plusOne expects one argument".to_string());
        }
        self.arg = Some(args[0].clone_executor(ctx));
        Ok(())
    }
    fn destroy(&mut self) {}
    fn get_name(&self) -> String {
        "plusOne".to_string()
    }
    fn clone_scalar_function(&self) -> Box<dyn ScalarFunctionExecutor> {
        Box::new(self.clone())
    }
}

#[tokio::test]
#[ignore = "Old SiddhiQL syntax not part of M1"]
async fn udf_invoked_in_query() {
    let mut manager = SiddhiManager::new();
    manager.add_scalar_function_factory("plusOne".to_string(), Box::new(PlusOneFn::default()));

    let app = "\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In select plusOne(v) as v insert into Out;\n";
    let runner = AppRunner::new_with_manager(manager, app, "Out").await;
    runner.send("In", vec![AttributeValue::Int(1)]);
    let out = runner.shutdown();
    assert_eq!(out, vec![vec![AttributeValue::Int(2)]]);
}
