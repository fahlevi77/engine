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
