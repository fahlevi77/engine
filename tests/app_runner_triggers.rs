#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::query_api::siddhi_app::SiddhiApp;
use siddhi_rust::query_api::definition::TriggerDefinition;
use siddhi_rust::query_api::expression::constant::TimeUtil;
use std::time::Duration;
use std::thread::sleep;

#[test]
fn start_trigger_emits_once() {
    let mut app = SiddhiApp::new("T1".to_string());
    app.add_trigger_definition(TriggerDefinition::id("TrigStream".to_string()).at("start".to_string()));
    let runner = AppRunner::new_from_api(app, "TrigStream");
    sleep(Duration::from_millis(50));
    let out = runner.shutdown();
    assert_eq!(out.len(), 1);
}

#[test]
fn periodic_trigger_emits() {
    let mut app = SiddhiApp::new("T2".to_string());
    let trig = TriggerDefinition::id("PTStream".to_string())
        .at_every_time_constant(TimeUtil::millisec(50)).unwrap();
    app.add_trigger_definition(trig);
    let runner = AppRunner::new_from_api(app, "PTStream");
    sleep(Duration::from_millis(130));
    let out = runner.shutdown();
    assert!(out.len() >= 2);
}
