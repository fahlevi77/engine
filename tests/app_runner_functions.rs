#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use eventflux_rust::core::event::value::AttributeValue;

// TODO: NOT PART OF M1 - Only ROUND function is in M1, not LOG/UPPER
// M1 Query 7 tests built-in functions but specifically uses ROUND function only.
// LOG and UPPER functions are not part of M1 implementation.
// M1 covers: Basic queries, Windows, Joins, GROUP BY, HAVING, ORDER BY, LIMIT, ROUND function
// Additional built-in functions (LOG, UPPER, etc.) will be implemented in Phase 2.
// See feat/grammar/GRAMMAR_STATUS.md for M1 feature list.
#[tokio::test]
#[ignore = "Requires LOG/UPPER functions - Not part of M1"]
async fn app_runner_builtin_functions() {
    // Converted to SQL syntax - built-in functions are part of M1
    let app = "\
        CREATE STREAM In (a DOUBLE);\n\
        SELECT LOG(a) as l, UPPER('abc') as u FROM In;\n";
    let runner = AppRunner::new(app, "Out").await;
    runner.send("In", vec![AttributeValue::Double(1.0)]);
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![vec![
            AttributeValue::Double(0.0),
            AttributeValue::String("ABC".to_string())
        ]]
    );
}
