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

/// Marker trait for compiled conditions used by tables.
pub trait CompiledCondition: Debug + Send + Sync {}

/// Marker trait for compiled update sets used by tables.
pub trait CompiledUpdateSet: Debug + Send + Sync {}

/// Simple wrapper implementing `CompiledCondition` for tables that do not
/// perform any special compilation.
#[derive(Debug, Clone)]
pub struct SimpleCompiledCondition(pub Expression);
impl CompiledCondition for SimpleCompiledCondition {}

/// Simple wrapper implementing `CompiledUpdateSet` for tables that do not
/// perform any special compilation.
#[derive(Debug, Clone)]
pub struct SimpleCompiledUpdateSet(pub UpdateSet);
impl CompiledUpdateSet for SimpleCompiledUpdateSet {}

/// Trait representing a table that can store rows of `AttributeValue`s.
pub trait Table: Debug + Send + Sync {
    /// Inserts a row into the table.
    fn insert(&self, values: &[AttributeValue]);

    /// Updates rows matching `old_values` with `new_values`.
    /// Returns `true` if any row was updated.
    fn update(&self, old_values: &[AttributeValue], new_values: &[AttributeValue]) -> bool;

    /// Deletes rows matching `values` from the table.
    /// Returns `true` if any row was removed.
    fn delete(&self, values: &[AttributeValue]) -> bool;

    /// Finds the first row matching `values` and returns a clone of it.
    fn find(&self, values: &[AttributeValue]) -> Option<Vec<AttributeValue>>;

    /// Returns `true` if the table contains the given row.
    fn contains(&self, values: &[AttributeValue]) -> bool;

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

    fn update(&self, old_values: &[AttributeValue], new_values: &[AttributeValue]) -> bool {
        let mut rows = self.rows.write().unwrap();
        let mut updated = false;
        for row in rows.iter_mut() {
            if row.as_slice() == old_values {
                *row = new_values.to_vec();
                updated = true;
            }
        }
        updated
    }

    fn delete(&self, values: &[AttributeValue]) -> bool {
        let mut rows = self.rows.write().unwrap();
        let original_len = rows.len();
        rows.retain(|row| row.as_slice() != values);
        original_len != rows.len()
    }

    fn find(&self, values: &[AttributeValue]) -> Option<Vec<AttributeValue>> {
        self.rows
            .read()
            .unwrap()
            .iter()
            .find(|row| row.as_slice() == values)
            .cloned()
    }

    fn contains(&self, values: &[AttributeValue]) -> bool {
        self.rows.read().unwrap().iter().any(|row| row == values)
    }

    fn compile_condition(&self, cond: Expression) -> Box<dyn CompiledCondition> {
        Box::new(SimpleCompiledCondition(cond))
    }

    fn compile_update_set(&self, us: UpdateSet) -> Box<dyn CompiledUpdateSet> {
        Box::new(SimpleCompiledUpdateSet(us))
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
