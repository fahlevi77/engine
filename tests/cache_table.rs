use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::table::{CacheTable, Table};

#[test]
fn test_cache_insert_and_eviction() {
    let table = CacheTable::new(2);
    let r1 = vec![AttributeValue::Int(1)];
    let r2 = vec![AttributeValue::Int(2)];
    let r3 = vec![AttributeValue::Int(3)];
    table.insert(&r1);
    table.insert(&r2);
    table.insert(&r3);
    // r1 should be evicted
    assert!(!table.contains(&r1));
    assert!(table.contains(&r2));
    assert!(table.contains(&r3));
}

#[test]
fn test_cache_update_delete_find() {
    let table = CacheTable::new(3);
    let r1 = vec![AttributeValue::Int(1)];
    table.insert(&r1);
    let r2 = vec![AttributeValue::Int(2)];
    assert!(table.update(&r1, &r2));
    assert!(!table.contains(&r1));
    assert!(table.contains(&r2));
    assert_eq!(table.find(&r2), Some(r2.clone()));
    assert!(table.delete(&r2));
    assert!(table.find(&r2).is_none());
}
