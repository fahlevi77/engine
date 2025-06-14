#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::core::event::value::AttributeValue;

#[test]
fn test_processor_pipeline() {
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
fn test_order_by_limit_offset() {
    use siddhi_rust::query_api::definition::{attribute::Type as AttrType, StreamDefinition};
    use siddhi_rust::query_api::execution::query::output::output_stream::{
        InsertIntoStreamAction, OutputStream, OutputStreamAction,
    };
    use siddhi_rust::query_api::execution::query::selection::order_by_attribute::Order;
    use siddhi_rust::query_api::execution::query::selection::Selector;
    use siddhi_rust::query_api::execution::query::{input::InputStream, Query};
    use siddhi_rust::query_api::execution::ExecutionElement;
    use siddhi_rust::query_api::expression::{constant::Constant, variable::Variable};
    use siddhi_rust::query_api::siddhi_app::SiddhiApp;

    let mut app = SiddhiApp::new("App".to_string());
    app.add_stream_definition(
        StreamDefinition::new("InputStream".to_string()).attribute("a".to_string(), AttrType::INT),
    );
    app.add_stream_definition(
        StreamDefinition::new("OutStream".to_string()).attribute("a".to_string(), AttrType::INT),
    );

    let input = InputStream::stream("InputStream".to_string());
    let selector = Selector::new()
        .select_variable(Variable::new("a".to_string()))
        .order_by_with_order(Variable::new("a".to_string()), Order::Desc)
        .limit(Constant::int(2))
        .unwrap()
        .offset(Constant::int(1))
        .unwrap();
    let out_action = InsertIntoStreamAction {
        target_id: "OutStream".to_string(),
        is_inner_stream: false,
        is_fault_stream: false,
    };
    let out_stream = OutputStream::new(OutputStreamAction::InsertInto(out_action), None);
    let query = Query::query()
        .from(input)
        .select(selector)
        .out_stream(out_stream);
    app.add_execution_element(ExecutionElement::Query(query));

    let runner = AppRunner::new_from_api(app, "OutStream");
    runner.send_batch(
        "InputStream",
        vec![
            vec![AttributeValue::Int(1)],
            vec![AttributeValue::Int(5)],
            vec![AttributeValue::Int(3)],
            vec![AttributeValue::Int(2)],
        ],
    );
    let out = runner.shutdown();
    let expected = vec![vec![AttributeValue::Int(3)], vec![AttributeValue::Int(2)]];
    assert_eq!(out, expected);
}

#[test]
fn test_group_by_order_by() {
    use siddhi_rust::query_api::definition::{attribute::Type as AttrType, StreamDefinition};
    use siddhi_rust::query_api::execution::query::output::output_stream::{
        InsertIntoStreamAction, OutputStream, OutputStreamAction,
    };
    use siddhi_rust::query_api::execution::query::selection::order_by_attribute::Order;
    use siddhi_rust::query_api::execution::query::selection::Selector;
    use siddhi_rust::query_api::execution::query::{input::InputStream, Query};
    use siddhi_rust::query_api::execution::ExecutionElement;
    use siddhi_rust::query_api::expression::variable::Variable;
    use siddhi_rust::query_api::siddhi_app::SiddhiApp;

    let mut app = SiddhiApp::new("App2".to_string());
    app.add_stream_definition(
        StreamDefinition::new("InputStream".to_string()).attribute("a".to_string(), AttrType::INT),
    );
    app.add_stream_definition(
        StreamDefinition::new("OutStream".to_string()).attribute("a".to_string(), AttrType::INT),
    );

    let input = InputStream::stream("InputStream".to_string());
    let selector = Selector::new()
        .select_variable(Variable::new("a".to_string()))
        .group_by(Variable::new("a".to_string()))
        .order_by_with_order(Variable::new("a".to_string()), Order::Asc);
    let out_action = InsertIntoStreamAction {
        target_id: "OutStream".to_string(),
        is_inner_stream: false,
        is_fault_stream: false,
    };
    let out_stream = OutputStream::new(OutputStreamAction::InsertInto(out_action), None);
    let query = Query::query()
        .from(input)
        .select(selector)
        .out_stream(out_stream);
    app.add_execution_element(ExecutionElement::Query(query));

    let runner = AppRunner::new_from_api(app, "OutStream");
    runner.send_batch(
        "InputStream",
        vec![
            vec![AttributeValue::Int(2)],
            vec![AttributeValue::Int(1)],
            vec![AttributeValue::Int(2)],
        ],
    );
    let out = runner.shutdown();
    let expected = vec![vec![AttributeValue::Int(1)], vec![AttributeValue::Int(2)]];
    assert_eq!(out, expected);
}
