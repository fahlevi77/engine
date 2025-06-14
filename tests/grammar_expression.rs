use siddhi_rust::query_api::expression::condition::compare::Operator as CompareOp;
use siddhi_rust::query_api::expression::Expression;
use siddhi_rust::query_compiler::parse_expression;

#[test]
fn test_arithmetic_precedence() {
    let expr = parse_expression("1 + 2 * 3").unwrap();
    let expected = Expression::add(
        Expression::value_long(1),
        Expression::multiply(Expression::value_long(2), Expression::value_long(3)),
    );
    assert_eq!(expr, expected);
}

#[test]
fn test_boolean_logic_precedence() {
    let expr = parse_expression("not a and b or c").unwrap();
    let expected = Expression::or(
        Expression::and(
            Expression::not(Expression::variable("a".to_string())),
            Expression::variable("b".to_string()),
        ),
        Expression::variable("c".to_string()),
    );
    assert_eq!(expr, expected);
}

#[test]
fn test_comparison_and_parentheses() {
    let expr = parse_expression("(a + 1) * 2 > b - 3").unwrap();
    let left = Expression::multiply(
        Expression::add(
            Expression::variable("a".to_string()),
            Expression::value_long(1),
        ),
        Expression::value_long(2),
    );
    let right = Expression::subtract(
        Expression::variable("b".to_string()),
        Expression::value_long(3),
    );
    let expected = Expression::compare(left, CompareOp::GreaterThan, right);
    assert_eq!(expr, expected);
}
