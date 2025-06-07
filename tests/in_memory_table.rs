use siddhi_rust::core::table::{InMemoryTable, Table};
use siddhi_rust::core::event::value::AttributeValue;

#[test]
fn test_insert_and_contains() {
    let table = InMemoryTable::new();
    let row = vec![
        AttributeValue::Int(1),
        AttributeValue::String("a".to_string()),
    ];
    table.insert(&row);
    assert!(table.contains(&row));
}

#[test]
fn test_contains_false() {
    let table = InMemoryTable::new();
    let row1 = vec![AttributeValue::Int(2)];
    let row2 = vec![AttributeValue::Int(3)];
    table.insert(&row1);
    assert!(!table.contains(&row2));
}

#[test]
fn test_update() {
    let table = InMemoryTable::new();
    let old = vec![AttributeValue::Int(1)];
    let new = vec![AttributeValue::Int(2)];
    table.insert(&old);
    assert!(table.update(&old, &new));
    assert!(!table.contains(&old));
    assert!(table.contains(&new));
}

#[test]
fn test_delete() {
    let table = InMemoryTable::new();
    let row1 = vec![AttributeValue::Int(1)];
    let row2 = vec![AttributeValue::Int(2)];
    table.insert(&row1);
    table.insert(&row2);
    assert!(table.delete(&row1));
    assert!(!table.contains(&row1));
    assert!(table.contains(&row2));
}

#[test]
fn test_find() {
    let table = InMemoryTable::new();
    let row = vec![AttributeValue::Int(42)];
    table.insert(&row);
    let found = table.find(&row);
    assert_eq!(found, Some(row.clone()));
    assert!(table.find(&[AttributeValue::Int(0)]).is_none());
}
