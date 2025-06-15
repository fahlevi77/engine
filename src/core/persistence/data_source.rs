// siddhi_rust/src/core/persistence/data_source.rs
use crate::core::config::siddhi_app_context::SiddhiAppContext;
use std::any::Any; // For connection object, highly abstract
use std::fmt::Debug;
use std::sync::Arc; // For init context

// Placeholder for actual configuration structure for a data source
#[derive(Debug, Clone, Default)]
pub struct DataSourceConfig {
    pub r#type: String, // e.g., "RDBMS", "InMemoryTable" - using r# to allow 'type' as field name
    pub properties: std::collections::HashMap<String, String>, // e.g., URL, username, password
}

pub trait DataSource: Debug + Send + Sync + 'static {
    // 'static for Arc<dyn DataSource>
    // Returns a string identifier for the type of data source (e.g., "RDBMS", "InMemory")
    fn get_type(&self) -> String;

    // Initializes the data source with configurations.
    // `configs` could be a map or a specific struct. Using String for now as in Java example.
    // `siddhi_app_name` is provided for context, e.g., for namespacing connections or tables.
    fn init(
        &mut self,
        siddhi_app_context: &Arc<SiddhiAppContext>,
        data_source_id: &str,
        config: DataSourceConfig,
    ) -> Result<(), String>;

    // Returns a connection object. The type of this object is specific to the DataSource implementation.
    // Using Box<dyn Any> to represent an arbitrary connection type.
    // Callers would need to downcast this to the expected concrete connection type.
    fn get_connection(&self) -> Result<Box<dyn Any>, String>;

    // Shuts down the data source and releases any resources.
    fn shutdown(&mut self) -> Result<(), String>;

    // For cloning the DataSource factory/template, if needed by SiddhiContext
    fn clone_data_source(&self) -> Box<dyn DataSource>;
}

// Helper for cloning Box<dyn DataSource>
impl Clone for Box<dyn DataSource> {
    fn clone(&self) -> Self {
        self.clone_data_source()
    }
}

// --- Concrete DataSource Implementations ---

use rusqlite::Connection;
use std::sync::Mutex;

/// Simple SQLite backed data source. Connections are shared via `Arc<Mutex<Connection>>`.
#[derive(Debug)]
pub struct SqliteDataSource {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteDataSource {
    /// Create a new SQLite data source backed by the file at `path`.
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            conn: Arc::new(Mutex::new(Connection::open(path)?)),
        })
    }
}

impl DataSource for SqliteDataSource {
    fn get_type(&self) -> String {
        "sqlite".to_string()
    }

    fn init(
        &mut self,
        _ctx: &Arc<SiddhiAppContext>,
        _id: &str,
        cfg: DataSourceConfig,
    ) -> Result<(), String> {
        if let Some(path) = cfg.properties.get("path") {
            let conn = Connection::open(path).map_err(|e| e.to_string())?;
            self.conn = Arc::new(Mutex::new(conn));
        }
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
