#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::core::event::value::AttributeValue;

#[test]
fn app_runner_builtin_functions() {
    let app = "\
        define stream In (a double);\n\
        define stream Out (l double, u string);\n\
        from In select log(a) as l, upper('abc') as u insert into Out;\n";
    let runner = AppRunner::new(app, "Out");
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
