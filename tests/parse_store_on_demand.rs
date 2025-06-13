use siddhi_rust::query_compiler::{parse_on_demand_query, parse_store_query};
use siddhi_rust::query_api::execution::query::{OnDemandQuery, OnDemandQueryType, StoreQuery};
use siddhi_rust::query_api::execution::query::input::store::InputStore;
use siddhi_rust::query_api::execution::query::selection::Selector;
use siddhi_rust::query_api::expression::variable::Variable;

#[test]
fn test_parse_on_demand_query_basic() {
    let parsed = parse_on_demand_query("from StockTable select symbol").unwrap();
    let expected = OnDemandQuery::query()
        .from(InputStore::Store(InputStore::store("StockTable".to_string())))
        .select(Selector::new().select_variable(Variable::new("symbol".to_string())))
        .set_type(OnDemandQueryType::Find);
    assert_eq!(parsed, expected);
}

#[test]
fn test_parse_store_query_basic() {
    let parsed = parse_store_query("from StockTable select symbol").unwrap();
    let expected_odq = OnDemandQuery::query()
        .from(InputStore::Store(InputStore::store("StockTable".to_string())))
        .select(Selector::new().select_variable(Variable::new("symbol".to_string())))
        .set_type(OnDemandQueryType::Find);
    let expected = StoreQuery::new(expected_odq);
    assert_eq!(parsed, expected);
}
