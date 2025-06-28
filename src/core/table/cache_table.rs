use crate::core::config::siddhi_context::SiddhiContext;
use crate::core::event::value::AttributeValue;
use crate::core::extension::TableFactory;
use crate::core::table::Table;
use crate::core::table::{CompiledCondition, CompiledUpdateSet, SimpleCompiledCondition, SimpleCompiledUpdateSet};
use crate::query_api::expression::Expression;
use crate::query_api::execution::query::output::stream::UpdateSet;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct CacheTable {
    rows: RwLock<VecDeque<Vec<AttributeValue>>>,
    max_size: usize,
}

impl CacheTable {
    pub fn new(max_size: usize) -> Self {
        Self {
            rows: RwLock::new(VecDeque::new()),
            max_size,
        }
    }

    fn trim_if_needed(&self, rows: &mut VecDeque<Vec<AttributeValue>>) {
        while rows.len() > self.max_size {
            rows.pop_front();
        }
    }

    /// Helper for tests to access all rows
    #[allow(dead_code)]
    pub fn all_rows(&self) -> Vec<Vec<AttributeValue>> {
        self.rows.read().unwrap().iter().cloned().collect()
    }
}

impl Table for CacheTable {
    fn insert(&self, values: &[AttributeValue]) {
        let mut rows = self.rows.write().unwrap();
        rows.push_back(values.to_vec());
        self.trim_if_needed(&mut rows);
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
        let orig_len = rows.len();
        rows.retain(|row| row.as_slice() != values);
        orig_len != rows.len()
    }

    fn find(&self, values: &[AttributeValue]) -> Option<Vec<AttributeValue>> {
        self
            .rows
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
        Box::new(CacheTable {
            rows: RwLock::new(rows),
            max_size: self.max_size,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CacheTableFactory;

impl TableFactory for CacheTableFactory {
    fn name(&self) -> &'static str {
        "cache"
    }

    fn create(
        &self,
        _name: String,
        mut properties: HashMap<String, String>,
        _ctx: Arc<SiddhiContext>,
    ) -> Result<Arc<dyn Table>, String> {
        let size_str = properties
            .remove("max_size")
            .ok_or_else(|| "max_size property required".to_string())?;
        let size = size_str
            .parse::<usize>()
            .map_err(|_| "max_size must be a number".to_string())?;
        Ok(Arc::new(CacheTable::new(size)))
    }

    fn clone_box(&self) -> Box<dyn TableFactory> {
        Box::new(self.clone())
    }
}

