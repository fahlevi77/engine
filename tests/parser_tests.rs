#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::core::event::value::AttributeValue;

#[test]
fn test_filter_projection() {
    let app = "\
        define stream InputStream (a int);\n\
        define stream OutStream (a int);\n\
        from InputStream[a > 10] select a insert into OutStream;\n";
    let runner = AppRunner::new(app, "OutStream");
    runner.send("InputStream", vec![AttributeValue::Int(5)]);
    runner.send("InputStream", vec![AttributeValue::Int(20)]);
    let out = runner.shutdown();
    assert_eq!(out, vec![vec![AttributeValue::Int(20)]]);
}

#[test]
fn test_length_window() {
    let app = "\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#length(2) select v insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send("In", vec![AttributeValue::Int(1)]);
    runner.send("In", vec![AttributeValue::Int(2)]);
    runner.send("In", vec![AttributeValue::Int(3)]);
    let out = runner.shutdown();
    assert_eq!(out, vec![
        vec![AttributeValue::Int(1)],
        vec![AttributeValue::Int(2)],
        vec![AttributeValue::Int(1)],
        vec![AttributeValue::Int(3)],
    ]);
}


#[test]
fn test_sum_aggregation() {
    let app = "\
        define stream InStream (v int);\n\
        define stream OutStream (total long);\n\
        from InStream select sum(v) as total insert into OutStream;\n";
    let runner = AppRunner::new(app, "OutStream");
    runner.send("InStream", vec![AttributeValue::Int(2)]);
    runner.send("InStream", vec![AttributeValue::Int(3)]);
    let out = runner.shutdown();
    assert_eq!(out, vec![vec![AttributeValue::Long(2)], vec![AttributeValue::Long(5)]]);
}

#[test]
fn test_join_query() {
    let app = "\
        define stream Left (a int);\n\
        define stream Right (b int);\n\
        define stream Out (a int, b int);\n\
        from Left join Right on a == a select a, Right.b insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send("Left", vec![AttributeValue::Int(5)]);
    runner.send("Right", vec![AttributeValue::Int(5)]);
    let out = runner.shutdown();
    assert_eq!(out, vec![vec![AttributeValue::Int(5), AttributeValue::Int(5)]]);
}

#[test]
fn test_builtin_function_in_query() {
    let app = "\
        define stream In (v string);\n\
        define stream Out (len int);\n\
        from In select str:length(v) as len insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send("In", vec![AttributeValue::String("abc".to_string())]);
    let out = runner.shutdown();
    assert_eq!(out, vec![vec![AttributeValue::Int(3)]]);
}

#[test]
fn test_udf_in_query() {
    use siddhi_rust::core::siddhi_manager::SiddhiManager;
    use siddhi_rust::core::executor::expression_executor::ExpressionExecutor;
    use siddhi_rust::core::executor::function::scalar_function_executor::ScalarFunctionExecutor;
    use siddhi_rust::query_api::definition::attribute::Type as AttrType;
    use siddhi_rust::core::config::siddhi_app_context::SiddhiAppContext;
    use siddhi_rust::query_compiler::parse;
    use siddhi_rust::query_api::siddhi_app::SiddhiApp;
    use std::sync::Arc;

    #[derive(Debug, Default)]
    struct PlusOneFn { arg: Option<Box<dyn ExpressionExecutor>> }

    impl Clone for PlusOneFn { fn clone(&self) -> Self { Self { arg: None } } }

    impl ExpressionExecutor for PlusOneFn {
        fn execute(&self, event: Option<&dyn siddhi_rust::core::event::complex_event::ComplexEvent>) -> Option<AttributeValue> {
            let v = self.arg.as_ref()?.execute(event)?;
            match v { AttributeValue::Int(i) => Some(AttributeValue::Int(i+1)), _ => None }
        }
        fn get_return_type(&self) -> AttrType { AttrType::INT }
        fn clone_executor(&self, _ctx:&Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> { Box::new(self.clone()) }
    }
    impl ScalarFunctionExecutor for PlusOneFn {
        fn init(&mut self, args:&Vec<Box<dyn ExpressionExecutor>>, ctx:&Arc<SiddhiAppContext>) -> Result<(), String> {
            if args.len()!=1 { return Err("plusOne expects one argument".to_string()); }
            self.arg = Some(args[0].clone_executor(ctx));
            Ok(())
        }
        fn destroy(&mut self) {}
        fn get_name(&self) -> String { "plusOne".to_string() }
        fn clone_scalar_function(&self) -> Box<dyn ScalarFunctionExecutor> { Box::new(self.clone()) }
    }

    let manager = SiddhiManager::new();
    manager.add_scalar_function_factory("plusOne".to_string(), Box::new(PlusOneFn::default()));

    let app_str = "\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In select plusOne(v) as v insert into Out;\n";
    use siddhi_rust::core::stream::output::stream_callback::StreamCallback;
    use siddhi_rust::core::event::event::Event;
    use std::sync::Mutex;

    #[derive(Debug)]
    struct CollectCB { ev: Arc<Mutex<Vec<Vec<AttributeValue>>>> }
    impl StreamCallback for CollectCB {
        fn receive_events(&self, events: &[Event]) { self.ev.lock().unwrap().extend(events.iter().map(|e| e.data.clone())); }
    }

    let api = parse(app_str).unwrap();
    let runtime = manager.create_siddhi_app_runtime_from_api(Arc::new(api), None).unwrap();
    let collected = Arc::new(Mutex::new(Vec::new()));
    runtime.add_callback("Out", Box::new(CollectCB { ev: Arc::clone(&collected) })).unwrap();
    runtime.start();
    let handler = runtime.get_input_handler("In").unwrap();
    handler.lock().unwrap().send_event_with_timestamp(0, vec![AttributeValue::Int(4)]).unwrap();
    runtime.shutdown();
    let out = collected.lock().unwrap().clone();
    assert_eq!(out, vec![vec![AttributeValue::Int(5)]]);
}
