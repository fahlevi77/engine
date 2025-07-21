#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::core::event::value::AttributeValue;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

#[test]
#[ignore]
fn async_partition_pool_order() {
    let app = "@app:async('true')\n\
        define stream In (v int, p string);\n\
        define stream Out (v int, p string);\n\
        define partition with (p of In) begin\n\
            from In select v, p insert into Out;\n\
        end;";
    let runner = AppRunner::new(app, "Out");
    for i in 0..5000 {
        let part = match i % 4 {
            0 => "a",
            1 => "b",
            2 => "c",
            _ => "d",
        };
        runner.send(
            "In",
            vec![
                AttributeValue::Int(i as i32),
                AttributeValue::String(part.to_string()),
            ],
        );
    }
    for _ in 0..20 {
        if runner.collected.lock().unwrap().len() >= 5000 {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    let out = runner.shutdown();
    assert_eq!(out.len(), 5000);
    let mut last: HashMap<String, i32> = HashMap::new();
    for row in out {
        if let (AttributeValue::Int(v), AttributeValue::String(ref p)) = (&row[0], &row[1]) {
            let e = last.entry(p.clone()).or_insert(-1);
            assert!(*v > *e);
            *e = *v;
        }
    }
}
