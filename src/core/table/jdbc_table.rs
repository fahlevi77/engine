use crate::core::config::siddhi_context::SiddhiContext;
use crate::core::event::stream::stream_event::StreamEvent;
use crate::core::event::value::AttributeValue;
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::table::{
    constant_to_av, CompiledCondition, CompiledUpdateSet, InMemoryCompiledCondition,
    InMemoryCompiledUpdateSet, Table,
};
use crate::query_api::execution::query::output::stream::UpdateSet;
use crate::query_api::expression::Expression;
use rusqlite::types::{Value, ValueRef};
use rusqlite::{params_from_iter, Connection};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct JdbcTable {
    table_name: String,
    data_source_name: String,
    siddhi_context: Arc<SiddhiContext>,
    conn: Arc<Mutex<Connection>>,
}

impl JdbcTable {
    pub fn new(
        table_name: String,
        data_source_name: String,
        siddhi_context: Arc<SiddhiContext>,
    ) -> Result<Self, String> {
        let ds = siddhi_context
            .get_data_source(&data_source_name)
            .ok_or_else(|| format!("DataSource '{data_source_name}' not found"))?;
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
            AttributeValue::Bool(b) => Value::Integer(if *b { 1 } else { 0 }),
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

#[derive(Debug, Clone)]
pub struct JdbcCompiledCondition {
    pub where_clause: String,
    pub params: Vec<AttributeValue>,
}

impl CompiledCondition for JdbcCompiledCondition {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct JdbcCompiledUpdateSet {
    pub assignments: String,
    pub params: Vec<AttributeValue>,
}

impl CompiledUpdateSet for JdbcCompiledUpdateSet {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone)]
pub enum JoinParamSource {
    Constant(AttributeValue),
    StreamAttr(usize),
}

#[derive(Debug, Clone)]
pub struct JdbcJoinCompiledCondition {
    pub where_clause: String,
    pub params: Vec<JoinParamSource>,
}

impl CompiledCondition for JdbcJoinCompiledCondition {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn expression_to_sql(expr: &Expression, params: &mut Vec<AttributeValue>) -> String {
    use crate::query_api::expression::condition::compare::Operator as CmpOp;
    use Expression::*;
    match expr {
        Constant(c) => {
            params.push(constant_to_av(c));
            "?".to_string()
        }
        Variable(v) => v.attribute_name.clone(),
        Compare(c) => {
            let left = expression_to_sql(&c.left_expression, params);
            let right = expression_to_sql(&c.right_expression, params);
            let op = match c.operator {
                CmpOp::LessThan => "<",
                CmpOp::GreaterThan => ">",
                CmpOp::LessThanEqual => "<=",
                CmpOp::GreaterThanEqual => ">=",
                CmpOp::Equal => "=",
                CmpOp::NotEqual => "!=",
            };
            format!("{left} {op} {right}")
        }
        And(a) => {
            let left = expression_to_sql(&a.left_expression, params);
            let right = expression_to_sql(&a.right_expression, params);
            format!("({left} AND {right})")
        }
        Or(o) => {
            let left = expression_to_sql(&o.left_expression, params);
            let right = expression_to_sql(&o.right_expression, params);
            format!("({left} OR {right})")
        }
        Not(n) => {
            let inner = expression_to_sql(&n.expression, params);
            format!("NOT ({inner})")
        }
        _ => String::new(),
    }
}

fn expression_to_sql_join(
    expr: &Expression,
    stream_id: &str,
    stream_def: &crate::query_api::definition::stream_definition::StreamDefinition,
    params: &mut Vec<JoinParamSource>,
) -> String {
    use crate::query_api::expression::condition::compare::Operator as CmpOp;
    use Expression::*;
    match expr {
        Constant(c) => {
            params.push(JoinParamSource::Constant(constant_to_av(c)));
            "?".to_string()
        }
        Variable(v) => {
            if v.stream_id.as_deref() == Some(stream_id) {
                if let Some(idx) = stream_def
                    .abstract_definition
                    .attribute_list
                    .iter()
                    .position(|a| a.get_name() == &v.attribute_name)
                {
                    params.push(JoinParamSource::StreamAttr(idx));
                    "?".to_string()
                } else {
                    v.attribute_name.clone()
                }
            } else {
                v.attribute_name.clone()
            }
        }
        Compare(c) => {
            let left = expression_to_sql_join(&c.left_expression, stream_id, stream_def, params);
            let right = expression_to_sql_join(&c.right_expression, stream_id, stream_def, params);
            let op = match c.operator {
                CmpOp::LessThan => "<",
                CmpOp::GreaterThan => ">",
                CmpOp::LessThanEqual => "<=",
                CmpOp::GreaterThanEqual => ">=",
                CmpOp::Equal => "=",
                CmpOp::NotEqual => "!=",
            };
            format!("{left} {op} {right}")
        }
        And(a) => {
            let left = expression_to_sql_join(&a.left_expression, stream_id, stream_def, params);
            let right = expression_to_sql_join(&a.right_expression, stream_id, stream_def, params);
            format!("({left} AND {right})")
        }
        Or(o) => {
            let left = expression_to_sql_join(&o.left_expression, stream_id, stream_def, params);
            let right = expression_to_sql_join(&o.right_expression, stream_id, stream_def, params);
            format!("({left} OR {right})")
        }
        Not(n) => {
            let inner = expression_to_sql_join(&n.expression, stream_id, stream_def, params);
            format!("NOT ({inner})")
        }
        _ => String::new(),
    }
}

impl Table for JdbcTable {
    fn insert(&self, values: &[AttributeValue]) {
        let placeholders = (0..values.len()).map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("INSERT INTO {} VALUES ({})", self.table_name, placeholders);
        let params: Vec<Value> = values.iter().map(Self::av_to_val).collect();
        let conn = self.conn.lock().unwrap();
        conn.execute(&sql, params_from_iter(params.iter())).unwrap();
    }

    fn all_rows(&self) -> Vec<Vec<AttributeValue>> {
        let sql = format!("SELECT * FROM {}", self.table_name);
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&sql).unwrap();
        let mut rows = stmt.query([]).unwrap();
        let mut out = Vec::new();
        while let Some(row) = rows.next().unwrap() {
            out.push(Self::row_to_attr(row));
        }
        out
    }

    fn update(
        &self,
        condition: &dyn CompiledCondition,
        update_set: &dyn CompiledUpdateSet,
    ) -> bool {
        if let (Some(cond), Some(us)) = (
            condition.as_any().downcast_ref::<JdbcCompiledCondition>(),
            update_set.as_any().downcast_ref::<JdbcCompiledUpdateSet>(),
        ) {
            let sql = format!(
                "UPDATE {} SET {} WHERE {}",
                self.table_name, us.assignments, cond.where_clause
            );
            let mut params: Vec<Value> = us.params.iter().map(Self::av_to_val).collect();
            params.extend(cond.params.iter().map(Self::av_to_val));
            let conn = self.conn.lock().unwrap();
            let count = conn.execute(&sql, params_from_iter(params.iter())).unwrap();
            return count > 0;
        }

        let cond_vals = match condition
            .as_any()
            .downcast_ref::<InMemoryCompiledCondition>()
        {
            Some(c) => &c.values,
            None => return false,
        };
        let us_vals = match update_set
            .as_any()
            .downcast_ref::<InMemoryCompiledUpdateSet>()
        {
            Some(u) => &u.values,
            None => return false,
        };
        let set_clause = (0..us_vals.len())
            .map(|i| format!("c{i}=?"))
            .collect::<Vec<_>>()
            .join(",");
        let where_clause = (0..cond_vals.len())
            .map(|i| format!("c{i}=?"))
            .collect::<Vec<_>>()
            .join(" AND ");
        let sql = format!(
            "UPDATE {} SET {} WHERE {}",
            self.table_name, set_clause, where_clause
        );
        let mut params: Vec<Value> = us_vals.iter().map(Self::av_to_val).collect();
        params.extend(cond_vals.iter().map(Self::av_to_val));
        let conn = self.conn.lock().unwrap();
        let count = conn.execute(&sql, params_from_iter(params.iter())).unwrap();
        count > 0
    }

    fn delete(&self, condition: &dyn CompiledCondition) -> bool {
        if let Some(cond) = condition.as_any().downcast_ref::<JdbcCompiledCondition>() {
            let sql = format!(
                "DELETE FROM {} WHERE {}",
                self.table_name, cond.where_clause
            );
            let params: Vec<Value> = cond.params.iter().map(Self::av_to_val).collect();
            let conn = self.conn.lock().unwrap();
            let count = conn.execute(&sql, params_from_iter(params.iter())).unwrap();
            return count > 0;
        }

        let values = match condition
            .as_any()
            .downcast_ref::<InMemoryCompiledCondition>()
        {
            Some(c) => &c.values,
            None => return false,
        };
        let where_clause = (0..values.len())
            .map(|i| format!("c{i}=?"))
            .collect::<Vec<_>>()
            .join(" AND ");
        let sql = format!("DELETE FROM {} WHERE {}", self.table_name, where_clause);
        let params: Vec<Value> = values.iter().map(Self::av_to_val).collect();
        let conn = self.conn.lock().unwrap();
        let count = conn.execute(&sql, params_from_iter(params.iter())).unwrap();
        count > 0
    }

    fn find(&self, condition: &dyn CompiledCondition) -> Option<Vec<AttributeValue>> {
        if let Some(cond) = condition.as_any().downcast_ref::<JdbcCompiledCondition>() {
            let sql = format!(
                "SELECT * FROM {} WHERE {} LIMIT 1",
                self.table_name, cond.where_clause
            );
            let params: Vec<Value> = cond.params.iter().map(Self::av_to_val).collect();
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare(&sql).unwrap();
            let mut rows = stmt.query(params_from_iter(params.iter())).unwrap();
            return rows.next().unwrap().map(|row| Self::row_to_attr(row));
        }

        let values = match condition
            .as_any()
            .downcast_ref::<InMemoryCompiledCondition>()
        {
            Some(c) => &c.values,
            None => return None,
        };
        let where_clause = (0..values.len())
            .map(|i| format!("c{i}=?"))
            .collect::<Vec<_>>()
            .join(" AND ");
        let sql = format!(
            "SELECT * FROM {} WHERE {} LIMIT 1",
            self.table_name, where_clause
        );
        let params: Vec<Value> = values.iter().map(Self::av_to_val).collect();
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&sql).unwrap();
        let mut rows = stmt.query(params_from_iter(params.iter())).unwrap();
        rows.next().unwrap().map(|row| Self::row_to_attr(row))
    }

    fn contains(&self, condition: &dyn CompiledCondition) -> bool {
        self.find(condition).is_some()
    }

    fn find_rows_for_join(
        &self,
        stream_event: &StreamEvent,
        compiled_condition: Option<&dyn CompiledCondition>,
        condition_executor: Option<&dyn ExpressionExecutor>,
    ) -> Vec<Vec<AttributeValue>> {
        if let Some(cond) =
            compiled_condition.and_then(|c| c.as_any().downcast_ref::<JdbcJoinCompiledCondition>())
        {
            let sql = if cond.where_clause.is_empty() {
                format!("SELECT * FROM {}", self.table_name)
            } else {
                format!(
                    "SELECT * FROM {} WHERE {}",
                    self.table_name, cond.where_clause
                )
            };
            let mut params: Vec<Value> = Vec::new();
            for p in &cond.params {
                match p {
                    JoinParamSource::Constant(av) => params.push(Self::av_to_val(av)),
                    JoinParamSource::StreamAttr(idx) => {
                        params.push(Self::av_to_val(&stream_event.before_window_data[*idx]))
                    }
                }
            }
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare(&sql).unwrap();
            let mut rows = stmt.query(params_from_iter(params.iter())).unwrap();
            let mut out = Vec::new();
            while let Some(row) = rows.next().unwrap() {
                out.push(Self::row_to_attr(row));
            }
            return out;
        }

        // Fallback to in-memory style evaluation
        let sql = format!("SELECT * FROM {}", self.table_name);
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&sql).unwrap();
        let mut rows_iter = stmt.query([]).unwrap();
        let mut matched = Vec::new();
        let stream_attr_count = stream_event.before_window_data.len();
        while let Some(row) = rows_iter.next().unwrap() {
            let vals = Self::row_to_attr(row);
            if let Some(exec) = condition_executor {
                let mut joined =
                    StreamEvent::new(stream_event.timestamp, stream_attr_count + vals.len(), 0, 0);
                for i in 0..stream_attr_count {
                    joined.before_window_data[i] = stream_event.before_window_data[i].clone();
                }
                for j in 0..vals.len() {
                    joined.before_window_data[stream_attr_count + j] = vals[j].clone();
                }
                if let Some(AttributeValue::Bool(true)) = exec.execute(Some(&joined)) {
                    matched.push(vals);
                }
            } else {
                matched.push(vals);
            }
        }
        matched
    }

    fn compile_condition(&self, cond: Expression) -> Box<dyn CompiledCondition> {
        let mut params = Vec::new();
        let where_clause = expression_to_sql(&cond, &mut params);
        Box::new(JdbcCompiledCondition {
            where_clause,
            params,
        })
    }

    fn compile_update_set(&self, us: UpdateSet) -> Box<dyn CompiledUpdateSet> {
        let mut params = Vec::new();
        let mut assigns = Vec::new();
        for sa in us.set_attributes.iter() {
            let expr_sql = expression_to_sql(&sa.value_to_set, &mut params);
            assigns.push(format!("{} = {}", sa.table_column.attribute_name, expr_sql));
        }
        Box::new(JdbcCompiledUpdateSet {
            assignments: assigns.join(","),
            params,
        })
    }

    fn compile_join_condition(
        &self,
        cond: Expression,
        stream_id: &str,
        stream_def: &crate::query_api::definition::stream_definition::StreamDefinition,
    ) -> Option<Box<dyn CompiledCondition>> {
        let mut params = Vec::new();
        let where_clause = expression_to_sql_join(&cond, stream_id, stream_def, &mut params);
        Some(Box::new(JdbcJoinCompiledCondition {
            where_clause,
            params,
        }))
    }

    fn clone_table(&self) -> Box<dyn Table> {
        Box::new(
            JdbcTable::new(
                self.table_name.clone(),
                self.data_source_name.clone(),
                Arc::clone(&self.siddhi_context),
            )
            .unwrap(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct JdbcTableFactory;

impl crate::core::extension::TableFactory for JdbcTableFactory {
    fn name(&self) -> &'static str {
        "jdbc"
    }
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
