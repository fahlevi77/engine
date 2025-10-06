// TODO: NOT PART OF M1 - All table tests disabled for M1
// Table support (DEFINE TABLE, @store annotations, cache/JDBC tables, stream-table joins)
// is not part of the M1 milestone. M1 focuses on:
// - Basic queries, Windows, Joins (stream-stream), GROUP BY, HAVING, ORDER BY, LIMIT
// Table infrastructure including cache tables, JDBC tables, and stream-table joins
// will be implemented in Phase 2.
// See feat/grammar/GRAMMAR_STATUS.md for M1 feature list.

#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use rusqlite::Connection;
use siddhi_rust::core::config::siddhi_context::SiddhiContext;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::persistence::data_source::{DataSource, DataSourceConfig};
use siddhi_rust::core::siddhi_manager::SiddhiManager;
use siddhi_rust::query_compiler::{parse_expression, parse_set_clause};
use std::any::Any;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct SqliteDataSource {
    conn: Arc<Mutex<Connection>>,
}
impl SqliteDataSource {
    fn new(path: &str) -> Self {
        Self {
            conn: Arc::new(Mutex::new(Connection::open(path).unwrap())),
        }
    }
}
impl DataSource for SqliteDataSource {
    fn get_type(&self) -> String {
        "sqlite".to_string()
    }
    fn init(
        &mut self,
        _ctx: &Arc<siddhi_rust::core::config::siddhi_app_context::SiddhiAppContext>,
        _id: &str,
        _cfg: DataSourceConfig,
    ) -> Result<(), String> {
        Ok(())
    }
    fn get_connection(&self) -> Result<Box<dyn Any>, String> {
        Ok(Box::new(self.conn.clone()) as Box<dyn Any>)
    }
    fn shutdown(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn clone_data_source(&self) -> Box<dyn DataSource> {
        Box::new(SqliteDataSource {
            conn: self.conn.clone(),
        })
    }
}

fn setup_sqlite_table(ctx: &Arc<SiddhiContext>, name: &str) {
    let ds = ctx.get_data_source("DS1").unwrap();
    let conn_any = ds.get_connection().unwrap();
    let conn_arc = conn_any.downcast::<Arc<Mutex<Connection>>>().unwrap();
    let mut conn = conn_arc.lock().unwrap();
    let sql = match name {
        "J2" => format!("CREATE TABLE {} (roomNo INTEGER, type TEXT)", name),
        _ => format!("CREATE TABLE {} (v TEXT)", name),
    };
    conn.execute(&sql, []).unwrap();
}

#[tokio::test]
#[ignore = "Table support not part of M1"]
async fn cache_table_crud_via_app_runner() {
    let query = "\
        define stream In (v string);\n\
        define stream Out (v string);\n\
        @store(type='cache', max_size='5')\n\
        define table T (v string);\n\
        from In select v insert into T;\n\
        from In select v insert into Out;\n";
    let runner = AppRunner::new(query, "Out").await;
    runner.send("In", vec![AttributeValue::String("a".into())]);
    std::thread::sleep(std::time::Duration::from_millis(50));

    let table = runner
        .runtime()
        .siddhi_app_context
        .get_siddhi_context()
        .get_table("T")
        .unwrap();
    let cond_expr = parse_expression("'a'").unwrap();
    let cond = table.compile_condition(cond_expr);
    assert!(table.contains(&*cond));
    let us = parse_set_clause("set v = 'b'").unwrap();
    let us_comp = table.compile_update_set(us);
    assert!(table.update(&*cond, &*us_comp));
    let cond_b_expr = parse_expression("'b'").unwrap();
    let cond_b = table.compile_condition(cond_b_expr);
    assert!(table.contains(&*cond_b));
    assert_eq!(
        table.find(&*cond_b),
        Some(vec![AttributeValue::String("b".into())])
    );
    assert!(table.delete(&*cond_b));
    assert!(!table.contains(&*cond_b));
    let _ = runner.shutdown();
}

#[tokio::test]
#[ignore = "Table support not part of M1"]
async fn jdbc_table_crud_via_app_runner() {
    let mut manager = SiddhiManager::new();
    manager
        .add_data_source(
            "DS1".to_string(),
            Arc::new(SqliteDataSource::new(":memory:")),
        )
        .unwrap();
    let ctx = manager.siddhi_context();
    setup_sqlite_table(&ctx, "J");

    let query = "\
        define stream In (v string);\n\
        define stream Out (v string);\n\
        @store(type='jdbc', data_source='DS1')\n\
        define table J (v string);\n\
        from In select v insert into J;\n\
        from In select v insert into Out;\n";
    let runner = AppRunner::new_with_manager(manager, query, "Out").await;
    runner.send("In", vec![AttributeValue::String("x".into())]);
    std::thread::sleep(std::time::Duration::from_millis(50));

    let ctx = runner.runtime().siddhi_app_context.get_siddhi_context();
    let table = ctx.get_table("J").unwrap();
    let cond_expr = parse_expression("v == 'x'").unwrap();
    let cond = table.compile_condition(cond_expr);
    assert!(table.contains(&*cond));
    let us = parse_set_clause("set v = 'y'").unwrap();
    let us_comp = table.compile_update_set(us);
    assert!(table.update(&*cond, &*us_comp));
    let cond_y_expr = parse_expression("v == 'y'").unwrap();
    let cond_y = table.compile_condition(cond_y_expr);
    assert_eq!(
        table.find(&*cond_y),
        Some(vec![AttributeValue::String("y".into())])
    );
    assert!(table.delete(&*cond_y));
    assert!(!table.contains(&*cond_y));
    let _ = runner.shutdown();
}

#[tokio::test]
#[ignore = "Table support not part of M1"]
async fn stream_table_join_basic() {
    let query = "\
        define stream L (roomNo int, val string);\n\
        @store(type='cache', max_size='5')\n\
        define table R (roomNo int, type string);\n\
        define stream Out (r int, t string, v string);\n\
        from L join R on L.roomNo == R.roomNo\n\
        select L.roomNo as r, R.type as t, L.val as v\n\
        insert into Out;\n";
    let runner = AppRunner::new(query, "Out").await;
    let th = runner.runtime().get_table_input_handler("R").unwrap();
    th.add(vec![siddhi_rust::core::event::event::Event::new_with_data(
        0,
        vec![AttributeValue::Int(1), AttributeValue::String("A".into())],
    )]);
    runner.send(
        "L",
        vec![AttributeValue::Int(1), AttributeValue::String("v".into())],
    );
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![vec![
            AttributeValue::Int(1),
            AttributeValue::String("A".into()),
            AttributeValue::String("v".into()),
        ]]
    );
}

#[tokio::test]
#[ignore = "Table support not part of M1"]
async fn stream_table_join_jdbc() {
    let mut manager = SiddhiManager::new();
    manager
        .add_data_source(
            "DS1".to_string(),
            Arc::new(SqliteDataSource::new(":memory:")),
        )
        .unwrap();
    let ctx = manager.siddhi_context();
    setup_sqlite_table(&ctx, "J2");

    let query = "\
        define stream L (roomNo int, val string);\n\
        @store(type='jdbc', data_source='DS1')\n\
        define table J2 (roomNo int, type string);\n\
        define stream Out (r int, t string, v string);\n\
        from L join J2 on L.roomNo == J2.roomNo\n\
        select L.roomNo as r, J2.type as t, L.val as v\n\
        insert into Out;\n";
    let runner = AppRunner::new_with_manager(manager, query, "Out").await;
    let th = runner.runtime().get_table_input_handler("J2").unwrap();
    th.add(vec![siddhi_rust::core::event::event::Event::new_with_data(
        0,
        vec![AttributeValue::Int(2), AttributeValue::String("B".into())],
    )]);
    runner.send(
        "L",
        vec![AttributeValue::Int(2), AttributeValue::String("x".into())],
    );
    let out = runner.shutdown();
    assert_eq!(
        out,
        vec![vec![
            AttributeValue::Int(2),
            AttributeValue::String("B".into()),
            AttributeValue::String("x".into()),
        ]]
    );
}

#[tokio::test]
#[ignore = "Table support not part of M1"]
async fn cache_and_jdbc_tables_eviction_and_queries() {
    let mut manager = SiddhiManager::new();
    manager
        .add_data_source(
            "DS1".to_string(),
            Arc::new(SqliteDataSource::new(":memory:")),
        )
        .unwrap();
    let ctx = manager.siddhi_context();
    setup_sqlite_table(&ctx, "J3");

    let query = "\
        define stream In (v string);\n\
        define stream Out (v string);\n\
        @store(type='cache', max_size='2')\n\
        define table C (v string);\n\
        @store(type='jdbc', data_source='DS1')\n\
        define table J3 (v string);\n\
        from In select v insert into C;\n\
        from In select v insert into J3;\n\
        from In select v insert into Out;\n";
    let runner = AppRunner::new_with_manager(manager, query, "Out").await;

    runner.send("In", vec![AttributeValue::String("a".into())]);
    runner.send("In", vec![AttributeValue::String("b".into())]);
    std::thread::sleep(std::time::Duration::from_millis(50));

    let ctx = runner.runtime().siddhi_app_context.get_siddhi_context();
    let cache = ctx.get_table("C").unwrap();
    let jdbc = ctx.get_table("J3").unwrap();

    let cond_a = cache.compile_condition(parse_expression("'a'").unwrap());
    assert!(cache.contains(&*cond_a));
    let us_x = cache.compile_update_set(parse_set_clause("set v = 'x'").unwrap());
    assert!(cache.update(&*cond_a, &*us_x));
    let cond_x = cache.compile_condition(parse_expression("'x'").unwrap());
    assert!(cache.contains(&*cond_x));

    let cond_b_j = jdbc.compile_condition(parse_expression("v == 'b'").unwrap());
    assert!(jdbc.contains(&*cond_b_j));
    let us_y = jdbc.compile_update_set(parse_set_clause("set v = 'y'").unwrap());
    assert!(jdbc.update(&*cond_b_j, &*us_y));
    let cond_y = jdbc.compile_condition(parse_expression("v == 'y'").unwrap());
    assert_eq!(
        jdbc.find(&*cond_y),
        Some(vec![AttributeValue::String("y".into())])
    );

    runner.send("In", vec![AttributeValue::String("c".into())]);
    std::thread::sleep(std::time::Duration::from_millis(50));

    let cond_b = cache.compile_condition(parse_expression("'b'").unwrap());
    let cond_c = cache.compile_condition(parse_expression("'c'").unwrap());
    assert!(cache.contains(&*cond_b));
    assert!(cache.contains(&*cond_c));
    assert!(!cache.contains(&*cond_x));

    assert!(jdbc.contains(&*cond_y));
    assert!(jdbc.delete(&*cond_y));
    assert!(!jdbc.contains(&*cond_y));
    let cond_a_j = jdbc.compile_condition(parse_expression("v == 'a'").unwrap());
    let cond_c_j = jdbc.compile_condition(parse_expression("v == 'c'").unwrap());
    assert!(jdbc.contains(&*cond_a_j));
    assert!(jdbc.contains(&*cond_c_j));

    let _ = runner.shutdown();
}
