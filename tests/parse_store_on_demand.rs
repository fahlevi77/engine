use eventflux_rust::query_api::execution::query::input::store::InputStore;
use eventflux_rust::query_api::execution::query::output::stream::UpdateSet;
use eventflux_rust::query_api::execution::query::selection::Selector;
use eventflux_rust::query_api::execution::query::{OnDemandQuery, OnDemandQueryType, StoreQuery};
use eventflux_rust::query_api::expression::condition::compare::Operator as CompareOperator;
use eventflux_rust::query_api::expression::variable::Variable;
use eventflux_rust::query_api::expression::Expression;
use eventflux_rust::query_compiler::{parse_on_demand_query, parse_store_query};

#[test]
fn test_parse_on_demand_query_basic() {
    let parsed = parse_on_demand_query("from StockTable select symbol").unwrap();
    let expected = OnDemandQuery::query()
        .from(InputStore::Store(InputStore::store(
            "StockTable".to_string(),
        )))
        .select(Selector::new().select_variable(Variable::new("symbol".to_string())))
        .set_type(OnDemandQueryType::Find);
    assert_eq!(parsed, expected);
}

#[test]
fn test_parse_store_query_basic() {
    let parsed = parse_store_query("from StockTable select symbol").unwrap();
    let expected_odq = OnDemandQuery::query()
        .from(InputStore::Store(InputStore::store(
            "StockTable".to_string(),
        )))
        .select(Selector::new().select_variable(Variable::new("symbol".to_string())))
        .set_type(OnDemandQueryType::Find);
    let expected = StoreQuery::new(expected_odq);
    assert_eq!(parsed, expected);
}

#[test]
fn test_parse_on_demand_delete() {
    let parsed = parse_on_demand_query("delete StockTable on symbol == 'EventFlux'").unwrap();
    let cond = Expression::compare(
        Expression::variable("symbol".to_string()),
        CompareOperator::Equal,
        Expression::value_string("EventFlux".to_string()),
    );
    let expected = OnDemandQuery::query()
        .delete_by("StockTable".to_string(), cond)
        .set_type(OnDemandQueryType::Delete);
    assert_eq!(parsed, expected);
}

#[test]
fn test_parse_on_demand_update_set() {
    let parsed =
        parse_on_demand_query("update StockTable set price = 0 on symbol == 'IBM'").unwrap();
    let mut set = UpdateSet::new();
    set = set.add_set_attribute(
        Variable::new("price".to_string()),
        Expression::value_long(0),
    );
    let cond = Expression::compare(
        Expression::variable("symbol".to_string()),
        CompareOperator::Equal,
        Expression::value_string("IBM".to_string()),
    );
    let expected = OnDemandQuery::query()
        .update_by_with_set("StockTable".to_string(), set, cond)
        .set_type(OnDemandQueryType::Update);
    assert_eq!(parsed, expected);
}
