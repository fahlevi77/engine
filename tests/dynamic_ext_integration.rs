// TODO: NOT PART OF M1 - Dynamic extension loading test
// This test uses old SiddhiQL syntax and tests dynamic extension loading.
// While extension system exists, SQL syntax for custom extensions is not in M1.
// M1 focuses on: Basic queries, Windows, Joins, GROUP BY, HAVING, ORDER BY, LIMIT
// Dynamic extension SQL syntax will be implemented in Phase 2.
// See feat/grammar/GRAMMAR_STATUS.md for M1 feature list.

#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::siddhi_manager::SiddhiManager;

#[tokio::test]
#[ignore = "Dynamic extension SQL syntax not part of M1"]
async fn test_dynamic_extension_loading() {
    let manager = SiddhiManager::new();
    let lib_path = custom_dyn_ext::library_path();
    manager
        .set_extension("dynlib", lib_path.to_str().unwrap().to_string())
        .unwrap();

    let ctx = manager.siddhi_context();
    assert!(ctx.get_window_factory("dynWindow").is_some());
    assert!(ctx.get_scalar_function_factory("dynPlusOne").is_some());

    let app = "\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#window:dynWindow() select dynPlusOne(v) as v insert into Out;\n";
    let runner = AppRunner::new_with_manager(manager, app, "Out").await;
    runner.send("In", vec![AttributeValue::Int(1)]);
    let out = runner.shutdown();
    assert_eq!(out, vec![vec![AttributeValue::Int(2)]]);
}
