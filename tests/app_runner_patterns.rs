#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::core::event::value::AttributeValue;

#[test]
fn sequence_basic() {
    let app = "\
        define stream AStream (val int);\n\
        define stream BStream (val int);\n\
        define stream OutStream (aval int, bval int);\n\
        from AStream -> BStream select AStream.val as aval, BStream.val as bval insert into OutStream;\n";
    let runner = AppRunner::new(app, "OutStream");
    runner.send("AStream", vec![AttributeValue::Int(1)]);
    runner.send("BStream", vec![AttributeValue::Int(2)]);
    runner.send("AStream", vec![AttributeValue::Int(3)]);
    runner.send("BStream", vec![AttributeValue::Int(4)]);
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![
            vec![AttributeValue::Int(1), AttributeValue::Int(2)],
            vec![AttributeValue::Int(3), AttributeValue::Int(4)],
        ]
    );
}

#[test]
fn every_sequence() {
    let app = "\
        define stream A (val int);\n\
        define stream B (val int);\n\
        define stream Out (aval int, bval int);\n\
        from every A -> B select A.val as aval, B.val as bval insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send("A", vec![AttributeValue::Int(1)]);
    runner.send("B", vec![AttributeValue::Int(2)]);
    runner.send("B", vec![AttributeValue::Int(3)]);
    runner.send("A", vec![AttributeValue::Int(4)]);
    runner.send("B", vec![AttributeValue::Int(5)]);
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![
            vec![AttributeValue::Int(1), AttributeValue::Int(2)],
            vec![AttributeValue::Int(1), AttributeValue::Int(3)],
            vec![AttributeValue::Int(4), AttributeValue::Int(5)],
        ]
    );
}

use siddhi_rust::query_api::definition::{attribute::Type as AttrType, StreamDefinition};
use std::sync::Arc;
use siddhi_rust::query_api::execution::query::input::state::State;
use siddhi_rust::query_api::execution::query::input::stream::input_stream::InputStream;
use siddhi_rust::query_api::execution::query::input::stream::single_input_stream::SingleInputStream;
use siddhi_rust::query_api::execution::query::input::stream::state_input_stream::StateInputStream;
use siddhi_rust::query_api::execution::query::output::output_stream::{InsertIntoStreamAction, OutputStream, OutputStreamAction};
use siddhi_rust::query_api::execution::query::selection::{OutputAttribute, Selector};
use siddhi_rust::query_api::execution::query::Query;
use siddhi_rust::query_api::execution::ExecutionElement;
use siddhi_rust::query_api::expression::{constant::TimeUtil, variable::Variable, Expression};

#[test]
fn kleene_star_pattern() {
    let mut app = siddhi_rust::query_api::siddhi_app::SiddhiApp::new("Kleene".to_string());
    let a_def = StreamDefinition::new("A".to_string()).attribute("val".to_string(), AttrType::INT);
    let b_def = StreamDefinition::new("B".to_string()).attribute("val".to_string(), AttrType::INT);
    let out_def = StreamDefinition::new("Out".to_string())
        .attribute("aval".to_string(), AttrType::INT)
        .attribute("bval".to_string(), AttrType::INT);
    app.stream_definition_map.insert("A".to_string(), Arc::new(a_def));
    app.stream_definition_map.insert("B".to_string(), Arc::new(b_def));
    app.stream_definition_map.insert("Out".to_string(), Arc::new(out_def));

    let a_si = SingleInputStream::new_basic("A".to_string(), false, false, None, Vec::new());
    let b_si = SingleInputStream::new_basic("B".to_string(), false, false, None, Vec::new());
    let pattern = State::next(
        State::zero_or_many(State::stream(a_si)),
        State::stream_element(b_si),
    );
    let state_stream = StateInputStream::sequence_stream(pattern, None);
    let input = InputStream::State(Box::new(state_stream));

    let mut selector = Selector::new();
    selector.selection_list = vec![
        OutputAttribute::new(
            Some("aval".to_string()),
            Expression::Variable(Variable::new("val".to_string()).of_stream("A".to_string())),
        ),
        OutputAttribute::new(
            Some("bval".to_string()),
            Expression::Variable(Variable::new("val".to_string()).of_stream("B".to_string())),
        ),
    ];
    let insert_action = InsertIntoStreamAction { target_id: "Out".to_string(), is_inner_stream: false, is_fault_stream: false };
    let out_stream = OutputStream::new(OutputStreamAction::InsertInto(insert_action), None);
    let query = Query::query().from(input).select(selector).out_stream(out_stream);
    app.execution_element_list.push(ExecutionElement::Query(query));

    let runner = AppRunner::new_from_api(app, "Out");
    runner.send("B", vec![AttributeValue::Int(1)]);
    let out = runner.shutdown();
    assert_eq!(out, vec![vec![AttributeValue::Null, AttributeValue::Int(1)]]);
}

#[test]
fn sequence_with_timeout() {
    let mut app = siddhi_rust::query_api::siddhi_app::SiddhiApp::new("Timeout".to_string());
    let a_def = StreamDefinition::new("A".to_string()).attribute("val".to_string(), AttrType::INT);
    let b_def = StreamDefinition::new("B".to_string()).attribute("val".to_string(), AttrType::INT);
    let out_def = StreamDefinition::new("Out".to_string())
        .attribute("aval".to_string(), AttrType::INT)
        .attribute("bval".to_string(), AttrType::INT);
    app.stream_definition_map.insert("A".to_string(), Arc::new(a_def));
    app.stream_definition_map.insert("B".to_string(), Arc::new(b_def));
    app.stream_definition_map.insert("Out".to_string(), Arc::new(out_def));

    let a_si = SingleInputStream::new_basic("A".to_string(), false, false, None, Vec::new());
    let b_si = SingleInputStream::new_basic("B".to_string(), false, false, None, Vec::new());
    let pattern = State::next(State::stream_element(a_si), State::stream_element(b_si));
    let state_stream = StateInputStream::sequence_stream(pattern, Some(TimeUtil::sec(1)));
    let input = InputStream::State(Box::new(state_stream));

    let mut selector = Selector::new();
    selector.selection_list = vec![
        OutputAttribute::new(
            Some("aval".to_string()),
            Expression::Variable(Variable::new("val".to_string()).of_stream("A".to_string())),
        ),
        OutputAttribute::new(
            Some("bval".to_string()),
            Expression::Variable(Variable::new("val".to_string()).of_stream("B".to_string())),
        ),
    ];
    let insert_action = InsertIntoStreamAction { target_id: "Out".to_string(), is_inner_stream: false, is_fault_stream: false };
    let out_stream = OutputStream::new(OutputStreamAction::InsertInto(insert_action), None);
    let query = Query::query().from(input).select(selector).out_stream(out_stream);
    app.execution_element_list.push(ExecutionElement::Query(query));

    let runner = AppRunner::new_from_api(app, "Out");
    runner.send_with_ts("A", 0, vec![AttributeValue::Int(1)]);
    runner.send_with_ts("B", 500, vec![AttributeValue::Int(2)]);
    runner.send_with_ts("A", 2000, vec![AttributeValue::Int(3)]);
    runner.send_with_ts("B", 3500, vec![AttributeValue::Int(4)]);
    let out = runner.shutdown();
    assert_eq!(out, vec![vec![AttributeValue::Int(1), AttributeValue::Int(2)]]);
}

