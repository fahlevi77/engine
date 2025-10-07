#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use eventflux_rust::core::event::value::AttributeValue;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// TODO: NOT PART OF M1 - Requires @app:async annotation support in SQL compiler
// This test uses @app:async annotation which is not part of core SQL syntax.
// M1 covers: Basic queries, Windows, Joins, GROUP BY, HAVING, ORDER BY, LIMIT
// Async annotations and advanced app-level annotations will be implemented in Phase 2.
// See feat/grammar/GRAMMAR_STATUS.md for M1 feature list.
#[tokio::test]
#[ignore = "Requires @app annotation support - Not part of M1"]
async fn async_junction_concurrent_dispatch() {
    let app = "@app:async('true')\n\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In select v insert into Out;\n";
    let runner = Arc::new(AppRunner::new(app, "Out").await);
    let mut handles = Vec::new();
    for i in 0..4 {
        let r = Arc::clone(&runner);
        handles.push(thread::spawn(move || {
            for k in 0..500 {
                r.send("In", vec![AttributeValue::Int(i * 500 + k)]);
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    thread::sleep(Duration::from_millis(300));
    let runner = Arc::try_unwrap(runner).unwrap();
    let out = runner.shutdown();
    assert_eq!(out.len(), 2000);
}
