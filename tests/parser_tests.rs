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
    assert_eq!(out, vec![vec![AttributeValue::Int(1)], vec![AttributeValue::Int(2)], vec![AttributeValue::Int(3)]]);
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
