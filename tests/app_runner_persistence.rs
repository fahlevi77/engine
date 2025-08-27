#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::persistence::{InMemoryPersistenceStore, PersistenceStore};
use std::sync::Arc;

#[tokio::test]
async fn persist_restore_no_error() {
    let store: Arc<dyn PersistenceStore> = Arc::new(InMemoryPersistenceStore::new());
    let app = "\
        @app:name('PersistApp')\n\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#length(2) select v insert into Out;\n";
    let runner = AppRunner::new_with_store(app, "Out", Arc::clone(&store)).await;
    runner.send("In", vec![AttributeValue::Int(1)]);
    let rev = runner.persist();
    runner.send("In", vec![AttributeValue::Int(2)]);
    // restore should succeed
    runner.restore_revision(&rev);
    let _ = runner.shutdown();
    assert!(!rev.is_empty());
}

#[tokio::test]
async fn length_window_restore_state() {
    let store: Arc<dyn PersistenceStore> = Arc::new(InMemoryPersistenceStore::new());
    let app = "\
        @app:name('PersistApp')\n\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#length(2) select v insert into Out;\n";
    let runner = AppRunner::new_with_store(app, "Out", Arc::clone(&store)).await;
    runner.send("In", vec![AttributeValue::Int(1)]);
    runner.send("In", vec![AttributeValue::Int(2)]);
    let rev = runner.persist();
    runner.send("In", vec![AttributeValue::Int(3)]);
    runner.restore_revision(&rev);
    runner.send("In", vec![AttributeValue::Int(4)]);
    let out = runner.shutdown();
    assert_eq!(out.last().unwrap(), &vec![AttributeValue::Int(4)]);
}

#[tokio::test]
async fn persist_shutdown_restore_state() {
    let store: Arc<dyn PersistenceStore> = Arc::new(InMemoryPersistenceStore::new());
    let app = "\
        @app:name('PersistApp')\n\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#length(2) select v insert into Out;\n";
    let runner = AppRunner::new_with_store(app, "Out", Arc::clone(&store)).await;
    runner.send("In", vec![AttributeValue::Int(1)]);
    runner.send("In", vec![AttributeValue::Int(2)]);
    let rev = runner.persist();
    runner.send("In", vec![AttributeValue::Int(3)]);
    let _ = runner.shutdown();

    let runner2 = AppRunner::new_with_store(app, "Out", Arc::clone(&store)).await;
    runner2.restore_revision(&rev);
    runner2.send("In", vec![AttributeValue::Int(4)]);
    let out = runner2.shutdown();
    assert_eq!(out.last().unwrap(), &vec![AttributeValue::Int(4)]);
}
