use crate::core::config::siddhi_context::SiddhiContext;
use crate::core::event::value::AttributeValue;
use crate::core::extension::TableFactory;
use crate::core::table::Table;
use crate::core::table::{
    constant_to_av, CompiledCondition, CompiledUpdateSet, InMemoryCompiledCondition,
    InMemoryCompiledUpdateSet,
};
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

    /// Update rows using already compiled condition and update set.
    pub fn update_compiled(
        &self,
        condition: &InMemoryCompiledCondition,
        update_set: &InMemoryCompiledUpdateSet,
    ) -> bool {
        let mut rows = self.rows.write().unwrap();
        let mut updated = false;
        for row in rows.iter_mut() {
            if row.as_slice() == condition.values.as_slice() {
                *row = update_set.values.clone();
                updated = true;
            }
        }
        updated
    }

    /// Delete rows matching the compiled condition.
    pub fn delete_compiled(&self, condition: &InMemoryCompiledCondition) -> bool {
        let mut rows = self.rows.write().unwrap();
        let orig_len = rows.len();
        rows.retain(|row| row.as_slice() != condition.values.as_slice());
        orig_len != rows.len()
    }

    /// Find a row matching the compiled condition.
    pub fn find_compiled(&self, condition: &InMemoryCompiledCondition) -> Option<Vec<AttributeValue>> {
        self
            .rows
            .read()
            .unwrap()
            .iter()
            .find(|row| row.as_slice() == condition.values.as_slice())
            .cloned()
    }

    /// Check if any row matches the compiled condition.
    pub fn contains_compiled(&self, condition: &InMemoryCompiledCondition) -> bool {
        self
            .rows
            .read()
            .unwrap()
            .iter()
            .any(|row| row.as_slice() == condition.values.as_slice())
    }
}

impl Table for CacheTable {
    fn insert(&self, values: &[AttributeValue]) {
        let mut rows = self.rows.write().unwrap();
        rows.push_back(values.to_vec());
        self.trim_if_needed(&mut rows);
    }

    fn update(
        &self,
        condition: &dyn CompiledCondition,
        update_set: &dyn CompiledUpdateSet,
    ) -> bool {
        match (
            condition.as_any().downcast_ref::<InMemoryCompiledCondition>(),
            update_set.as_any().downcast_ref::<InMemoryCompiledUpdateSet>(),
        ) {
            (Some(cond), Some(us)) => self.update_compiled(cond, us),
            _ => false,
        }
    }

    fn delete(&self, condition: &dyn CompiledCondition) -> bool {
        match condition.as_any().downcast_ref::<InMemoryCompiledCondition>() {
            Some(cond) => self.delete_compiled(cond),
            None => false,
        }
    }

    fn find(&self, condition: &dyn CompiledCondition) -> Option<Vec<AttributeValue>> {
        match condition.as_any().downcast_ref::<InMemoryCompiledCondition>() {
            Some(cond) => self.find_compiled(cond),
            None => None,
        }
    }

    fn contains(&self, condition: &dyn CompiledCondition) -> bool {
        match condition.as_any().downcast_ref::<InMemoryCompiledCondition>() {
            Some(cond) => self.contains_compiled(cond),
            None => false,
        }
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

