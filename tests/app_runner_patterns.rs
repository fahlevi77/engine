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
            vec![AttributeValue::Int(4), AttributeValue::Int(3)],
        ]
    );
}

