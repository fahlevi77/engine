#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::siddhi_manager::SiddhiManager;
use std::thread;
use std::time::Duration;

#[test]
#[ignore]
fn partition_async_ordered() {
    let app = "@app:async('true')\n\
        define stream In (v int, p string);\n\
        define stream Out (v int, p string);\n\
        define partition with (p of In) begin\n\
            from In select v, p insert into Out;\n\
        end;";
    let manager = SiddhiManager::new();
    let runner = AppRunner::new_with_manager(manager, app, "Out");
    for i in 0..1000 {
        let p = if i % 2 == 0 { "a" } else { "b" };
        runner.send(
            "In",
            vec![
                AttributeValue::Int(i as i32),
                AttributeValue::String(p.to_string()),
            ],
        );
    }
    thread::sleep(Duration::from_millis(1000));
    let out = runner.shutdown();
    assert_eq!(out.len(), 1000);
    let mut last_a = -1;
    let mut last_b = -1;
    for row in out {
        let v = match row[0] {
            AttributeValue::Int(i) => i,
            _ => continue,
        };
        let part = match &row[1] {
            AttributeValue::String(ref s) => s.as_str(),
            _ => "",
        };
        if part == "a" {
            assert!(v > last_a);
            last_a = v;
        } else {
            assert!(v > last_b);
            last_b = v;
        }
    }
}
