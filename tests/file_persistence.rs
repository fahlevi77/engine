#[path = "common/mod.rs"]
mod common;
use common::AppRunner;
use siddhi_rust::core::event::value::AttributeValue;
use siddhi_rust::core::persistence::{
    FilePersistenceStore, PersistenceStore, SqlitePersistenceStore,
};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn persist_restore_file_store() {
    let dir = tempdir().unwrap();
    let store: Arc<dyn PersistenceStore> = Arc::new(FilePersistenceStore::new(dir.path()).unwrap());
    let app = "\
        @app:name('PersistApp')\n\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#length(2) select v insert into Out;\n";
    let runner = AppRunner::new_with_store(app, "Out", Arc::clone(&store)).await;
    runner.send("In", vec![AttributeValue::Int(1)]);
    let rev = runner.persist();
    runner.send("In", vec![AttributeValue::Int(2)]);
    runner.restore_revision(&rev);
    let _ = runner.shutdown();
    assert!(!rev.is_empty());
}

#[tokio::test]
async fn persist_restore_sqlite_store() {
    let file = tempfile::NamedTempFile::new().unwrap();
    let store: Arc<dyn PersistenceStore> =
        Arc::new(SqlitePersistenceStore::new(file.path()).unwrap());
    let app = "\
        @app:name('PersistApp')\n\
        define stream In (v int);\n\
        define stream Out (v int);\n\
        from In#length(2) select v insert into Out;\n";
    let runner = AppRunner::new_with_store(app, "Out", Arc::clone(&store)).await;
    runner.send("In", vec![AttributeValue::Int(1)]);
    let rev = runner.persist();
    runner.send("In", vec![AttributeValue::Int(2)]);
    runner.restore_revision(&rev);
    let _ = runner.shutdown();
    assert!(!rev.is_empty());
}
