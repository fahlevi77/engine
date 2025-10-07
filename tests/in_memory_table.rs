use eventflux_rust::core::event::value::AttributeValue;
use eventflux_rust::core::table::{
    InMemoryCompiledCondition, InMemoryCompiledUpdateSet, InMemoryTable, Table,
};

#[test]
fn test_insert_and_contains() {
    let table = InMemoryTable::new();
    let row = vec![
        AttributeValue::Int(1),
        AttributeValue::String("a".to_string()),
    ];
    table.insert(&row);
    assert!(table.contains(&InMemoryCompiledCondition {
        values: row.clone()
    }));
}

#[test]
fn test_contains_false() {
    let table = InMemoryTable::new();
    let row1 = vec![AttributeValue::Int(2)];
    let row2 = vec![AttributeValue::Int(3)];
    table.insert(&row1);
    assert!(!table.contains(&InMemoryCompiledCondition { values: row2 }));
}

#[test]
fn test_update() {
    let table = InMemoryTable::new();
    let old = vec![AttributeValue::Int(1)];
    let new = vec![AttributeValue::Int(2)];
    table.insert(&old);
    let cond = InMemoryCompiledCondition {
        values: old.clone(),
    };
    let us = InMemoryCompiledUpdateSet {
        values: new.clone(),
    };
    assert!(table.update(&cond, &us));
    assert!(!table.contains(&InMemoryCompiledCondition { values: old }));
    assert!(table.contains(&InMemoryCompiledCondition { values: new }));
}

#[test]
fn test_delete() {
    let table = InMemoryTable::new();
    let row1 = vec![AttributeValue::Int(1)];
    let row2 = vec![AttributeValue::Int(2)];
    table.insert(&row1);
    table.insert(&row2);
    assert!(table.delete(&InMemoryCompiledCondition {
        values: row1.clone()
    }));
    assert!(!table.contains(&InMemoryCompiledCondition { values: row1 }));
    assert!(table.contains(&InMemoryCompiledCondition {
        values: row2.clone()
    }));
}

#[test]
fn test_find() {
    let table = InMemoryTable::new();
    let row = vec![AttributeValue::Int(42)];
    table.insert(&row);
    let found = table.find(&InMemoryCompiledCondition {
        values: row.clone(),
    });
    assert_eq!(found, Some(row.clone()));
    assert!(table
        .find(&InMemoryCompiledCondition {
            values: vec![AttributeValue::Int(0)]
        })
        .is_none());
}
