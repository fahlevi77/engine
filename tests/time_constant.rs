use siddhi_rust::query_api::expression::constant::TimeUtil;
use siddhi_rust::query_compiler::parse_time_constant;

#[test]
fn test_parse_seconds() {
    let c = parse_time_constant("5 sec").unwrap();
    assert_eq!(c, TimeUtil::sec(5));
}

#[test]
fn test_parse_minutes() {
    let c = parse_time_constant("2 min").unwrap();
    assert_eq!(c, TimeUtil::minute(2));
}

#[test]
fn test_parse_hours() {
    let c = parse_time_constant("1 hour").unwrap();
    assert_eq!(c, TimeUtil::hour(1));
}
