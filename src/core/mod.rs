// siddhi_rust/src/core/mod.rs

// Top-level files from Java io.siddhi.core
pub mod siddhi_app_runtime; // For SiddhiAppRuntime.java (and Impl)
pub mod siddhi_manager;     // For SiddhiManager.java
pub mod siddhi_app_runtime_builder; // Declare the module

// Sub-packages, corresponding to Java packages
pub mod aggregation;
pub mod config;
pub mod debugger;
pub mod event;
pub mod exception; // For custom core-specific error types
pub mod executor;
pub mod function;  // For UDFs like Script.java
pub mod partition;
pub mod query;
pub mod stream;
pub mod table;
pub mod trigger;
pub mod util;
pub mod window;
pub mod persistence; // Added
pub mod extension;

// Re-export key public-facing structs from core
pub use self::siddhi_app_runtime::SiddhiAppRuntime;
pub use self::siddhi_manager::SiddhiManager;
pub use self::siddhi_app_runtime_builder::SiddhiAppRuntimeBuilder;
pub use self::persistence::{DataSource, DataSourceConfig}; // Added
// Other important re-exports will be added as these modules are built out.
