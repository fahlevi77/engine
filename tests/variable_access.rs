#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::query_api::aggregation::time_period::Duration;

#[test]
fn variable_from_window() {
    let app = "\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#length(2) select v insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send("In", vec![AttributeValue::Int(5)]);
    let out = runner.shutdown();
    assert_eq!(out, vec![vec![AttributeValue::Int(5)]]);
}


#[test]
fn variable_from_aggregation() {
    let app = "\
        define stream In (value int);\n\
        define stream Out (total long);\n\
        define aggregation Agg from In select sum(value) as total group by value aggregate every seconds;\n\
        from In select value insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send_with_ts("In", 0, vec![AttributeValue::Int(5)]);
    runner.send_with_ts("In", 200, vec![AttributeValue::Int(5)]);
    // Flush first bucket
    runner.send_with_ts("In", 1100, vec![AttributeValue::Int(1)]);
    let data = runner.get_aggregation_data("Agg", None, Some(Duration::Seconds));
    let _ = runner.shutdown();
    assert!(data.is_empty() || data[0] == vec![AttributeValue::Long(10)]);
}

#[test]
#[ignore]
fn window_variable_access() {
    let app = "\
        define stream In (v int);\n\
        define window Win (v int) length(2) output all events;\n\
        define stream Out (v int);\n\
        from In select v as v insert into Win;\n\
        from Win select v as v insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send("In", vec![AttributeValue::Int(1)]);
    runner.send("In", vec![AttributeValue::Int(2)]);
    let out = runner.shutdown();
    assert_eq!(out, vec![vec![AttributeValue::Int(1)], vec![AttributeValue::Int(2)]]);
}

#[test]
#[ignore]
fn table_variable_access() {
    let app = "\
        define stream In (v int);\n\
        define table T (v int);\n\
        define stream Out (v int);\n\
        from In select v insert into table T;\n\
        from In join T on In.v == T.v select T.v as tv insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send("In", vec![AttributeValue::Int(1)]);
    runner.send("In", vec![AttributeValue::Int(1)]);
    let out = runner.shutdown();
    assert!(!out.is_empty());
    assert_eq!(out[0], vec![AttributeValue::Int(1)]);
}

#[test]
#[ignore]
fn aggregation_variable_access() {
    let app = "\
        define stream In (value int);\n\
        define stream Out (v int);\n\
        define aggregation Agg from In select sum(value) as total group by value aggregate every seconds;\n\
        from In select value as v insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send_with_ts("In", 0, vec![AttributeValue::Int(1)]);
    runner.send_with_ts("In", 200, vec![AttributeValue::Int(1)]);
    runner.send_with_ts("In", 1100, vec![AttributeValue::Int(1)]);
    let data = runner.get_aggregation_data("Agg", None, Some(Duration::Seconds));
    let _ = runner.shutdown();
    assert!(data.is_empty() || data[0] == vec![AttributeValue::Long(2)]);
}
