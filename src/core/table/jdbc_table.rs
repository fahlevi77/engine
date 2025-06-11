use crate::core::table::{Table};
use crate::core::event::value::AttributeValue;
use crate::core::config::siddhi_context::SiddhiContext;
use rusqlite::{Connection, params_from_iter};
use rusqlite::types::{Value, ValueRef};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct JdbcTable {
    table_name: String,
    data_source_name: String,
    siddhi_context: Arc<SiddhiContext>,
    conn: Arc<Mutex<Connection>>,
}

impl JdbcTable {
    pub fn new(table_name: String, data_source_name: String, siddhi_context: Arc<SiddhiContext>) -> Result<Self, String> {
        let ds = siddhi_context
            .get_data_source(&data_source_name)
            .ok_or_else(|| format!("DataSource '{}' not found", data_source_name))?;
        let conn_any = ds.get_connection()?;
        let conn_arc = match conn_any.downcast::<Arc<Mutex<Connection>>>() {
            Ok(c) => *c,
            Err(conn_any) => {
                let conn_box = conn_any
                    .downcast::<Connection>()
                    .map_err(|_| "Invalid connection type".to_string())?;
                Arc::new(Mutex::new(*conn_box))
            }
        };
        Ok(Self {
            table_name,
            data_source_name,
            siddhi_context,
            conn: conn_arc,
        })
    }

    fn av_to_val(av: &AttributeValue) -> Value {
        match av {
            AttributeValue::String(s) => Value::Text(s.clone()),
            AttributeValue::Int(i) => Value::Integer(*i as i64),
            AttributeValue::Long(l) => Value::Integer(*l),
            AttributeValue::Float(f) => Value::Real(*f as f64),
            AttributeValue::Double(d) => Value::Real(*d),
            AttributeValue::Bool(b) => Value::Integer(if *b {1} else {0}),
            AttributeValue::Null => Value::Null,
            AttributeValue::Object(_) => Value::Null,
        }
    }

    fn row_to_attr(row: &rusqlite::Row) -> Vec<AttributeValue> {
        (0..row.as_ref().column_count())
            .map(|i| match row.get_ref_unwrap(i) {
                ValueRef::Null => AttributeValue::Null,
                ValueRef::Integer(v) => AttributeValue::Long(v),
                ValueRef::Real(v) => AttributeValue::Double(v),
                ValueRef::Text(v) => AttributeValue::String(String::from_utf8_lossy(v).to_string()),
                ValueRef::Blob(_) => AttributeValue::Null,
            })
            .collect()
    }
}

impl Table for JdbcTable {
    fn insert(&self, values: &[AttributeValue]) {
        let placeholders = (0..values.len()).map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("INSERT INTO {} VALUES ({})", self.table_name, placeholders);
        let params: Vec<Value> = values.iter().map(Self::av_to_val).collect();
        let mut conn = self.conn.lock().unwrap();
        conn.execute(&sql, params_from_iter(params.iter())).unwrap();
    }

    fn update(&self, old_values: &[AttributeValue], new_values: &[AttributeValue]) -> bool {
        let set_clause = (0..new_values.len()).map(|i| format!("c{}=?", i)).collect::<Vec<_>>().join(",");
        let where_clause = (0..old_values.len()).map(|i| format!("c{}=?", i)).collect::<Vec<_>>().join(" AND ");
        let sql = format!("UPDATE {} SET {} WHERE {}", self.table_name, set_clause, where_clause);
        let mut params: Vec<Value> = new_values.iter().map(Self::av_to_val).collect();
        params.extend(old_values.iter().map(Self::av_to_val));
        let mut conn = self.conn.lock().unwrap();
        let count = conn.execute(&sql, params_from_iter(params.iter())).unwrap();
        count > 0
    }

    fn delete(&self, values: &[AttributeValue]) -> bool {
        let where_clause = (0..values.len()).map(|i| format!("c{}=?", i)).collect::<Vec<_>>().join(" AND ");
        let sql = format!("DELETE FROM {} WHERE {}", self.table_name, where_clause);
        let params: Vec<Value> = values.iter().map(Self::av_to_val).collect();
        let mut conn = self.conn.lock().unwrap();
        let count = conn.execute(&sql, params_from_iter(params.iter())).unwrap();
        count > 0
    }

    fn find(&self, values: &[AttributeValue]) -> Option<Vec<AttributeValue>> {
        let where_clause = (0..values.len()).map(|i| format!("c{}=?", i)).collect::<Vec<_>>().join(" AND ");
        let sql = format!("SELECT * FROM {} WHERE {} LIMIT 1", self.table_name, where_clause);
        let params: Vec<Value> = values.iter().map(Self::av_to_val).collect();
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&sql).unwrap();
        let mut rows = stmt.query(params_from_iter(params.iter())).unwrap();
        rows.next().unwrap().map(|row| Self::row_to_attr(row))
    }

    fn contains(&self, values: &[AttributeValue]) -> bool {
        self.find(values).is_some()
    }

    fn clone_table(&self) -> Box<dyn Table> {
        Box::new(JdbcTable::new(
            self.table_name.clone(),
            self.data_source_name.clone(),
            Arc::clone(&self.siddhi_context),
        ).unwrap())
    }
}

#[derive(Debug, Clone)]
pub struct JdbcTableFactory;

impl crate::core::extension::TableFactory for JdbcTableFactory {
    fn create(
        &self,
        table_name: String,
        mut properties: std::collections::HashMap<String, String>,
        ctx: Arc<SiddhiContext>,
    ) -> Result<Arc<dyn Table>, String> {
        let ds = properties
            .remove("data_source")
            .ok_or_else(|| "data_source property required".to_string())?;
        Ok(Arc::new(JdbcTable::new(table_name, ds, ctx)?))
    }

    fn clone_box(&self) -> Box<dyn crate::core::extension::TableFactory> {
        Box::new(self.clone())
    }
}
