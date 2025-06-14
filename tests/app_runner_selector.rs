#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::core::event::value::AttributeValue;

#[test]
fn group_by_having_order_limit_offset() {
    let app = "\
        define stream In (a int, b int);\n\
        define stream Out (b int, s long);\n\
        from In select b, sum(a) as s group by b having sum(a) > 5 order by b desc limit 2 offset 1 insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
    runner.send_batch(
        "In",
        vec![
            vec![AttributeValue::Int(3), AttributeValue::Int(1)],
            vec![AttributeValue::Int(4), AttributeValue::Int(1)],
            vec![AttributeValue::Int(10), AttributeValue::Int(2)],
            vec![AttributeValue::Int(1), AttributeValue::Int(3)],
        ],
    );
    let out = runner.shutdown();
    assert_eq!(out, vec![vec![AttributeValue::Int(1), AttributeValue::Long(7)]]);
}

