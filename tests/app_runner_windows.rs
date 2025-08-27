#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::core::event::value::AttributeValue;
use std::thread::sleep;
use std::time::Duration;

#[tokio::test]
async fn filter_projection_simple() {
    let app = "\
        define stream In (a int);\n\
        define stream Out (a int);\n\
        from In[a > 10] select a insert into Out;\n";
    let runner = AppRunner::new(app, "Out").await;
    runner.send("In", vec![AttributeValue::Int(5)]);
    runner.send("In", vec![AttributeValue::Int(15)]);
    let out = runner.shutdown();
    assert_eq!(out, vec![vec![AttributeValue::Int(15)]]);
}

#[tokio::test]
async fn length_window_basic() {
    let app = "\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#length(2) select v insert into Out;\n";
    let runner = AppRunner::new(app, "Out").await;
    runner.send("In", vec![AttributeValue::Int(1)]);
    runner.send("In", vec![AttributeValue::Int(2)]);
    runner.send("In", vec![AttributeValue::Int(3)]);
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![
            vec![AttributeValue::Int(1)],
            vec![AttributeValue::Int(2)],
            vec![AttributeValue::Int(1)],
            vec![AttributeValue::Int(3)],
        ]
    );
}

#[tokio::test]
async fn length_window_batch() {
    let app = "\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#length(2) select v insert into Out;\n";
    let runner = AppRunner::new(app, "Out").await;
    runner.send_batch(
        "In",
        vec![
            vec![AttributeValue::Int(1)],
            vec![AttributeValue::Int(2)],
            vec![AttributeValue::Int(3)],
        ],
    );
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![
            vec![AttributeValue::Int(1)],
            vec![AttributeValue::Int(2)],
            vec![AttributeValue::Int(1)],
            vec![AttributeValue::Int(3)],
        ]
    );
}

#[tokio::test]
async fn time_window_expiry() {
    let app = "\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#time(100) select v insert into Out;\n";
    let runner = AppRunner::new(app, "Out").await;
    runner.send("In", vec![AttributeValue::Int(5)]);
    sleep(Duration::from_millis(150));
    let out = runner.shutdown();
    assert!(out.len() >= 2);
    assert_eq!(out[0], vec![AttributeValue::Int(5)]);
}

#[tokio::test]
async fn length_batch_window() {
    let app = "\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#lengthBatch(2) select v insert into Out;\n";
    let runner = AppRunner::new(app, "Out").await;
    runner.send("In", vec![AttributeValue::Int(1)]);
    runner.send("In", vec![AttributeValue::Int(2)]);
    runner.send("In", vec![AttributeValue::Int(3)]);
    runner.send("In", vec![AttributeValue::Int(4)]);
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![
            vec![AttributeValue::Int(1)],
            vec![AttributeValue::Int(2)],
            vec![AttributeValue::Int(1)],
            vec![AttributeValue::Int(2)],
            vec![AttributeValue::Int(3)],
            vec![AttributeValue::Int(4)],
        ]
    );
}

#[tokio::test]
async fn time_batch_window() {
    let app = "\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#timeBatch(100) select v insert into Out;\n";
    let runner = AppRunner::new(app, "Out").await;
    runner.send("In", vec![AttributeValue::Int(1)]);
    sleep(Duration::from_millis(120));
    runner.send("In", vec![AttributeValue::Int(2)]);
    sleep(Duration::from_millis(120));
    let out = runner.shutdown();
    assert!(out.len() >= 3);
    assert_eq!(out[0], vec![AttributeValue::Int(1)]);
}

#[tokio::test]
async fn external_time_window_basic() {
    let app = "\
        define stream In (ts long, v int);\n\
        define stream Out (v int);\n\
        from In#externalTime(ts, 100) select v insert into Out;\n";
    let runner = AppRunner::new(app, "Out").await;
    runner.send_with_ts(
        "In",
        0,
        vec![AttributeValue::Long(0), AttributeValue::Int(1)],
    );
    runner.send_with_ts(
        "In",
        150,
        vec![AttributeValue::Long(150), AttributeValue::Int(2)],
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

#[tokio::test]
async fn external_time_batch_window() {
    let app = "\
        define stream In (ts long, v int);\n\
        define stream Out (v int);\n\
        from In#externalTimeBatch(ts, 100) select v insert into Out;\n";
    let runner = AppRunner::new(app, "Out").await;
    runner.send_with_ts(
        "In",
        0,
        vec![AttributeValue::Long(0), AttributeValue::Int(1)],
    );
    runner.send_with_ts(
        "In",
        60,
        vec![AttributeValue::Long(60), AttributeValue::Int(2)],
    );
    runner.send_with_ts(
        "In",
        120,
        vec![AttributeValue::Long(120), AttributeValue::Int(3)],
    );
    runner.send_with_ts(
        "In",
        240,
        vec![AttributeValue::Long(240), AttributeValue::Int(4)],
    );
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![
            vec![AttributeValue::Int(1)],
            vec![AttributeValue::Int(2)],
            vec![AttributeValue::Int(1)],
            vec![AttributeValue::Int(2)],
            vec![AttributeValue::Int(3)],
        ]
    );
}

#[tokio::test]
async fn lossy_counting_window() {
    let app = "\
        define stream In (v string);\n\
        define stream Out (v string);\n\
        from In#lossyCounting(1,1) select v insert into Out;\n";
    let runner = AppRunner::new(app, "Out").await;
    runner.send("In", vec![AttributeValue::String("A".to_string())]);
    runner.send("In", vec![AttributeValue::String("B".to_string())]);
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![
            vec![AttributeValue::String("A".to_string())],
            vec![AttributeValue::String("B".to_string())],
        ]
    );
}

#[tokio::test]
async fn cron_window_basic() {
    let app = "\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#cron('*/1 * * * * *') select v insert into Out;\n";
    let runner = AppRunner::new(app, "Out").await;
    runner.send("In", vec![AttributeValue::Int(1)]);
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let out = runner.shutdown();
    assert!(!out.is_empty());
    assert_eq!(out[0], vec![AttributeValue::Int(1)]);
}
