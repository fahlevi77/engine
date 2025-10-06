#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::core::event::value::AttributeValue;

// TODO: NOT PART OF M1 - PARTITION feature not in M1
// This test uses "partition with" syntax which is an advanced feature not included in M1.
// M1 covers: Basic queries, Windows, Joins, GROUP BY, HAVING, ORDER BY, LIMIT
// Partition support will be implemented in Phase 2.
// See feat/grammar/GRAMMAR_STATUS.md for M1 feature list.
#[tokio::test]
#[ignore = "Requires PARTITION support - Not part of M1"]
async fn partition_forward() {
    let app = "\
        define stream InStream (symbol string, volume int);\n\
        define stream OutStream (vol int);\n\
        partition with (symbol of InStream) begin \n\
            from InStream select volume as vol insert into OutStream; \n\
        end;\n";
    let runner = AppRunner::new(app, "OutStream").await;
    runner.send(
        "InStream",
        vec![AttributeValue::String("a".into()), AttributeValue::Int(1)],
    );
    runner.send(
        "InStream",
        vec![AttributeValue::String("b".into()), AttributeValue::Int(2)],
    );
    runner.send(
        "InStream",
        vec![AttributeValue::String("a".into()), AttributeValue::Int(3)],
    );
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![
            vec![AttributeValue::Int(1)],
            vec![AttributeValue::Int(2)],
            vec![AttributeValue::Int(3)],
        ]
    );
}

// TODO: NOT PART OF M1 - PARTITION feature not in M1
// This test uses "partition with" syntax which is an advanced feature not included in M1.
// M1 covers: Basic queries, Windows, Joins, GROUP BY, HAVING, ORDER BY, LIMIT
// Partition support will be implemented in Phase 2.
// See feat/grammar/GRAMMAR_STATUS.md for M1 feature list.
#[tokio::test]
#[ignore = "Requires PARTITION support - Not part of M1"]
async fn partition_sum_by_symbol() {
    let app = "\
        define stream InStream (symbol string, volume int);\n\
        define stream OutStream (sumvol long);\n\
        partition with (symbol of InStream) begin \n\
            from InStream select sum(volume) as sumvol insert into OutStream; \n\
        end;\n";
    let runner = AppRunner::new(app, "OutStream").await;
    runner.send(
        "InStream",
        vec![AttributeValue::String("x".into()), AttributeValue::Int(1)],
    );
    runner.send(
        "InStream",
        vec![AttributeValue::String("x".into()), AttributeValue::Int(2)],
    );
    runner.send(
        "InStream",
        vec![AttributeValue::String("y".into()), AttributeValue::Int(3)],
    );
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![
            vec![AttributeValue::Long(1)],
            vec![AttributeValue::Long(3)],
            vec![AttributeValue::Long(6)],
        ]
    );
}

// TODO: NOT PART OF M1 - PARTITION feature not in M1
// This test uses "partition with" syntax which is an advanced feature not included in M1.
// M1 covers: Basic queries, Windows, Joins, GROUP BY, HAVING, ORDER BY, LIMIT
// Partition support will be implemented in Phase 2.
// See feat/grammar/GRAMMAR_STATUS.md for M1 feature list.
#[tokio::test]
#[ignore = "Requires PARTITION support - Not part of M1"]
async fn partition_join_streams() {
    let app = "\
        define stream A (symbol string, v int);\n\
        define stream B (symbol string, v int);\n\
        define stream Out (a int, b int);\n\
        partition with (symbol of A, symbol of B) begin \n\
            from A join B on A.symbol == B.symbol select A.v as a, B.v as b insert into Out;\n\
        end;\n";
    let runner = AppRunner::new(app, "Out").await;
    runner.send(
        "A",
        vec![AttributeValue::String("s".into()), AttributeValue::Int(1)],
    );
    runner.send(
        "B",
        vec![AttributeValue::String("s".into()), AttributeValue::Int(2)],
    );
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![vec![AttributeValue::Int(1), AttributeValue::Int(2)]]
    );
}

// TODO: NOT PART OF M1 - PARTITION feature not in M1
// This test uses "partition with" syntax which is an advanced feature not included in M1.
// M1 covers: Basic queries, Windows, Joins, GROUP BY, HAVING, ORDER BY, LIMIT
// Partition support will be implemented in Phase 2.
// See feat/grammar/GRAMMAR_STATUS.md for M1 feature list.
#[tokio::test]
#[ignore = "Requires PARTITION support - Not part of M1"]
async fn partition_with_window() {
    let app = "\
        define stream In (symbol string, v int);\n\
        define stream Out (v int);\n\
        partition with (symbol of In) begin \n\
            from In#window:length(1) select v insert into Out;\n\
        end;\n";
    let runner = AppRunner::new(app, "Out").await;
    runner.send(
        "In",
        vec![AttributeValue::String("p".into()), AttributeValue::Int(1)],
    );
    runner.send(
        "In",
        vec![AttributeValue::String("p".into()), AttributeValue::Int(2)],
    );
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![
            vec![AttributeValue::Int(1)],
            vec![AttributeValue::Int(1)],
            vec![AttributeValue::Int(2)],
        ]
    );
}
