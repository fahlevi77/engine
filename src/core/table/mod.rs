use crate::core::event::value::AttributeValue;
use crate::query_api::execution::query::output::stream::UpdateSet;
use crate::query_api::expression::Expression;
use std::sync::RwLock;

mod jdbc_table;
mod cache_table;
use crate::core::config::siddhi_context::SiddhiContext;
use crate::core::extension::TableFactory;
pub use jdbc_table::{JdbcTable, JdbcTableFactory};
pub use cache_table::{CacheTable, CacheTableFactory};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use std::any::Any;

use crate::query_api::expression::constant::{Constant, ConstantValueWithFloat};

pub(crate) fn constant_to_av(c: &Constant) -> AttributeValue {
    match c.get_value() {
        ConstantValueWithFloat::String(s) => AttributeValue::String(s.clone()),
        ConstantValueWithFloat::Int(i) => AttributeValue::Int(*i),
        ConstantValueWithFloat::Long(l) => AttributeValue::Long(*l),
        ConstantValueWithFloat::Float(f) => AttributeValue::Float(*f),
        ConstantValueWithFloat::Double(d) => AttributeValue::Double(*d),
        ConstantValueWithFloat::Bool(b) => AttributeValue::Bool(*b),
        ConstantValueWithFloat::Time(t) => AttributeValue::Long(*t),
    }
}

/// Marker trait for compiled conditions used by tables.
pub trait CompiledCondition: Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

/// Marker trait for compiled update sets used by tables.
pub trait CompiledUpdateSet: Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

/// Simple wrapper implementing `CompiledCondition` for tables that do not
/// perform any special compilation.
#[derive(Debug, Clone)]
pub struct SimpleCompiledCondition(pub Expression);
impl CompiledCondition for SimpleCompiledCondition {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Simple wrapper implementing `CompiledUpdateSet` for tables that do not
/// perform any special compilation.
#[derive(Debug, Clone)]
pub struct SimpleCompiledUpdateSet(pub UpdateSet);
impl CompiledUpdateSet for SimpleCompiledUpdateSet {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Compiled condition representation used by [`InMemoryTable`] and [`CacheTable`].
#[derive(Debug, Clone)]
pub struct InMemoryCompiledCondition {
    /// Row of values that must match exactly.
    pub values: Vec<AttributeValue>,
}
impl CompiledCondition for InMemoryCompiledCondition {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Compiled update set representation used by [`InMemoryTable`] and [`CacheTable`].
#[derive(Debug, Clone)]
pub struct InMemoryCompiledUpdateSet {
    /// New values that should replace a matching row.
    pub values: Vec<AttributeValue>,
}
impl CompiledUpdateSet for InMemoryCompiledUpdateSet {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Trait representing a table that can store rows of `AttributeValue`s.
pub trait Table: Debug + Send + Sync {
    /// Inserts a row into the table.
    fn insert(&self, values: &[AttributeValue]);

    /// Updates rows matching `condition` using the values from `update_set`.
    /// Returns `true` if any row was updated.
    fn update(
        &self,
        condition: &dyn CompiledCondition,
        update_set: &dyn CompiledUpdateSet,
    ) -> bool;

    /// Deletes rows matching `condition` from the table.
    /// Returns `true` if any row was removed.
    fn delete(&self, condition: &dyn CompiledCondition) -> bool;

    /// Finds the first row matching `condition` and returns a clone of it.
    fn find(&self, condition: &dyn CompiledCondition) -> Option<Vec<AttributeValue>>;

    /// Returns `true` if the table contains any row matching `condition`.
    fn contains(&self, condition: &dyn CompiledCondition) -> bool;

    /// Compile a conditional expression into a table-specific representation.
    ///
    /// By default this wraps the expression in [`SimpleCompiledCondition`].
    fn compile_condition(&self, cond: Expression) -> Box<dyn CompiledCondition> {
        Box::new(SimpleCompiledCondition(cond))
    }

    /// Compile an update set into a table-specific representation.
    ///
    /// By default this wraps the update set in [`SimpleCompiledUpdateSet`].
    fn compile_update_set(&self, us: UpdateSet) -> Box<dyn CompiledUpdateSet> {
        Box::new(SimpleCompiledUpdateSet(us))
    }

    /// Clone helper for boxed trait objects.
    fn clone_table(&self) -> Box<dyn Table>;
}

impl Clone for Box<dyn Table> {
    fn clone(&self) -> Self {
        self.clone_table()
    }
}

/// Simple in-memory table storing rows in a vector.
#[derive(Debug, Default)]
pub struct InMemoryTable {
    rows: RwLock<Vec<Vec<AttributeValue>>>,
}

impl InMemoryTable {
    pub fn new() -> Self {
        Self {
            rows: RwLock::new(Vec::new()),
        }
    }

    pub fn all_rows(&self) -> Vec<Vec<AttributeValue>> {
        self.rows.read().unwrap().clone()
    }
}

impl Table for InMemoryTable {
    fn insert(&self, values: &[AttributeValue]) {
        self.rows.write().unwrap().push(values.to_vec());
    }

    fn update(
        &self,
        condition: &dyn CompiledCondition,
        update_set: &dyn CompiledUpdateSet,
    ) -> bool {
        let cond = match condition.as_any().downcast_ref::<InMemoryCompiledCondition>() {
            Some(c) => c,
            None => return false,
        };
        let us = match update_set.as_any().downcast_ref::<InMemoryCompiledUpdateSet>() {
            Some(u) => u,
            None => return false,
        };

        let mut rows = self.rows.write().unwrap();
        let mut updated = false;
        for row in rows.iter_mut() {
            if row.as_slice() == cond.values.as_slice() {
                *row = us.values.clone();
                updated = true;
            }
        }
        updated
    }

    fn delete(&self, condition: &dyn CompiledCondition) -> bool {
        let cond = match condition.as_any().downcast_ref::<InMemoryCompiledCondition>() {
            Some(c) => c,
            None => return false,
        };
        let mut rows = self.rows.write().unwrap();
        let orig_len = rows.len();
        rows.retain(|row| row.as_slice() != cond.values.as_slice());
        orig_len != rows.len()
    }

    fn find(&self, condition: &dyn CompiledCondition) -> Option<Vec<AttributeValue>> {
        let cond = match condition.as_any().downcast_ref::<InMemoryCompiledCondition>() {
            Some(c) => c,
            None => return None,
        };
        self.rows
            .read()
            .unwrap()
            .iter()
            .find(|row| row.as_slice() == cond.values.as_slice())
            .cloned()
    }

    fn contains(&self, condition: &dyn CompiledCondition) -> bool {
        let cond = match condition.as_any().downcast_ref::<InMemoryCompiledCondition>() {
            Some(c) => c,
            None => return false,
        };
        self.rows
            .read()
            .unwrap()
            .iter()
            .any(|row| row.as_slice() == cond.values.as_slice())
    }

    fn compile_condition(&self, cond: Expression) -> Box<dyn CompiledCondition> {
        if let Expression::Constant(c) = cond {
            Box::new(InMemoryCompiledCondition {
                values: vec![constant_to_av(&c)],
            })
        } else {
            Box::new(InMemoryCompiledCondition { values: Vec::new() })
        }
    }

    fn compile_update_set(&self, us: UpdateSet) -> Box<dyn CompiledUpdateSet> {
        let mut vals = Vec::new();
        for sa in us.set_attributes.iter() {
            if let Expression::Constant(c) = &sa.value_to_set {
                vals.push(constant_to_av(c));
            }
        }
        Box::new(InMemoryCompiledUpdateSet { values: vals })
    }

    fn clone_table(&self) -> Box<dyn Table> {
        let rows = self.rows.read().unwrap().clone();
        Box::new(InMemoryTable {
            rows: RwLock::new(rows),
        })
    }
}

#[derive(Debug, Clone)]
pub struct InMemoryTableFactory;

impl TableFactory for InMemoryTableFactory {
    fn name(&self) -> &'static str { "inMemory" }
    fn create(
        &self,
        _name: String,
        _properties: HashMap<String, String>,
        _ctx: Arc<SiddhiContext>,
    ) -> Result<Arc<dyn Table>, String> {
        Ok(Arc::new(InMemoryTable::new()))
    }

    fn clone_box(&self) -> Box<dyn TableFactory> {
        Box::new(self.clone())
    }
}
