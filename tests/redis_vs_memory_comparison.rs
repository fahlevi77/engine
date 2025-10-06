// tests/redis_vs_memory_comparison.rs

//! Direct comparison test between InMemory and Redis persistence stores
//! to isolate where the problem lies.

// TODO: NOT PART OF M1 - All tests in this file use old SiddhiQL syntax
// Tests use @app:name annotations and "define stream" which are not supported by SQL parser.
// See feat/grammar/GRAMMAR_STATUS.md for M1 feature list.

#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::persistence::{InMemoryPersistenceStore, RedisPersistenceStore, PersistenceStore};
use siddhi_rust::core::distributed::RedisConfig;
use std::sync::Arc;

#[tokio::test]
#[ignore = "@app:name annotation and old SiddhiQL syntax not part of M1"]
async fn test_memory_store_simple_works() {
    // Test the exact same pattern as the working app_runner_persistence test
    let store: Arc<dyn PersistenceStore> = Arc::new(InMemoryPersistenceStore::new());
    let app = "\
        @app:name('TestApp')\n\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#window:length(2) select v insert into Out;\n";
    
    let runner = AppRunner::new_with_store(app, "Out", Arc::clone(&store)).await;
    runner.send("In", vec![AttributeValue::Int(1)]);
    runner.send("In", vec![AttributeValue::Int(2)]);
    let rev = runner.persist();
    println!("Memory store revision: {}", rev);
    runner.send("In", vec![AttributeValue::Int(3)]);
    let _ = runner.shutdown();
    
    let runner2 = AppRunner::new_with_store(app, "Out", Arc::clone(&store)).await;
    runner2.restore_revision(&rev);
    runner2.send("In", vec![AttributeValue::Int(4)]);
    let out = runner2.shutdown();
    
    println!("Memory store simple output: {:?}", out);
    assert_eq!(out.last().unwrap(), &vec![AttributeValue::Int(4)]);
    println!("✅ Memory store simple pattern works!");
}

#[tokio::test]
#[ignore = "@app:name annotation and old SiddhiQL syntax not part of M1"]
async fn test_memory_store_with_count_fails() {
    let store: Arc<dyn PersistenceStore> = Arc::new(InMemoryPersistenceStore::new());
    let app = "\
        @app:name('TestApp')\n\
        define stream In (v int);\n\
        define stream Out (v int, count long);\n\
        from In#window:length(3) select v, count() as count insert into Out;\n";
    
    let runner = AppRunner::new_with_store(app, "Out", Arc::clone(&store)).await;
    
    // Build window state
    runner.send("In", vec![AttributeValue::Int(1)]);
    runner.send("In", vec![AttributeValue::Int(2)]);
    runner.send("In", vec![AttributeValue::Int(3)]);
    
    let rev = runner.persist();
    println!("Memory store revision: {}", rev);
    
    // Send more events after checkpoint
    runner.send("In", vec![AttributeValue::Int(4)]);
    
    // Restore from checkpoint
    runner.restore_revision(&rev);
    
    // Send new event - should have count=4 if restored correctly
    runner.send("In", vec![AttributeValue::Int(5)]);
    
    let out = runner.shutdown();
    
    // Verify the last event has the correct count
    if let Some(last_event) = out.last() {
        if let Some(AttributeValue::Long(count)) = last_event.get(1) {
            println!("Memory store final count: {}", count);
            if *count == 4 {
                println!("✅ Memory store with count works!");
            } else {
                println!("❌ Memory store with count fails: got count={}, expected count=4", count);
            }
        } else {
            panic!("Memory store: Expected count field in output");
        }
    } else {
        panic!("Memory store: Expected at least one output event");
    }
}

#[tokio::test]
#[ignore = "@app:name annotation and old SiddhiQL syntax not part of M1"]
async fn test_redis_store_debug() {
    let config = RedisConfig {
        url: "redis://localhost:6379".to_string(),
        max_connections: 5,
        connection_timeout_ms: 1000,
        key_prefix: "test:debug:".to_string(),
        ttl_seconds: Some(300),
    };
    
    let store: Arc<dyn PersistenceStore> = match RedisPersistenceStore::new_with_config(config) {
        Ok(store) => Arc::new(store),
        Err(_) => {
            println!("Redis not available, skipping test");
            return;
        }
    };
    
    let app = "\
        @app:name('TestApp')\n\
        define stream In (v int);\n\
        define stream Out (v int, count long);\n\
        from In#window:length(3) select v, count() as count insert into Out;\n";
    
    let runner = AppRunner::new_with_store(app, "Out", Arc::clone(&store)).await;
    
    // Build window state
    runner.send("In", vec![AttributeValue::Int(1)]);
    runner.send("In", vec![AttributeValue::Int(2)]);
    runner.send("In", vec![AttributeValue::Int(3)]);
    
    let rev = runner.persist();
    println!("Redis store revision: {}", rev);
    
    // Check what was actually saved to Redis
    if let Some(data) = store.load("TestApp", &rev) {
        println!("Redis store saved {} bytes", data.len());
        println!("First 50 bytes: {:?}", &data[..std::cmp::min(50, data.len())]);
    } else {
        println!("Redis store: No data found for revision {}", rev);
    }
    
    // Send more events after checkpoint
    runner.send("In", vec![AttributeValue::Int(4)]);
    
    // Restore from checkpoint
    runner.restore_revision(&rev);
    
    // Send new event - should have count=4 if restored correctly
    runner.send("In", vec![AttributeValue::Int(5)]);
    
    let out = runner.shutdown();
    
    // Verify the last event has the correct count
    if let Some(last_event) = out.last() {
        if let Some(AttributeValue::Long(count)) = last_event.get(1) {
            println!("Redis store final count: {}", count);
            // This will likely fail, but let's see what count we get
            if *count == 4 {
                println!("✅ Redis store works correctly!");
            } else {
                println!("❌ Redis store failed: got count={}, expected count=4", count);
            }
        } else {
            panic!("Redis store: Expected count field in output");
        }
    } else {
        panic!("Redis store: Expected at least one output event");
    }
}