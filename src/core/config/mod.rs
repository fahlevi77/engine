//! # Configuration Management
//! 
//! This module provides comprehensive configuration management for Siddhi applications.
//! 
//! ## Configuration Reload Policy
//! 
//! **Note**: Hot reload functionality is not supported in this implementation. 
//! 
//! To apply configuration changes:
//! 1. Enable state persistence in your Siddhi application
//! 2. Gracefully shutdown the current runtime 
//! 3. Update configuration files or environment variables
//! 4. Restart the application with new configuration
//! 5. State will be automatically restored, ensuring continuity
//! 
//! This approach maintains data consistency and reliable state recovery.

// Legacy context modules
pub mod siddhi_app_context;
pub mod siddhi_context;
pub mod siddhi_on_demand_query_context;
pub mod siddhi_query_context;
pub mod statistics_configuration;

// New configuration management modules
pub mod types;
pub mod manager;
pub mod loader;
pub mod validator;
pub mod resolver;
pub mod security;
pub mod service_discovery;
pub mod monitoring;
pub mod validation_api;
pub mod error;
pub mod processor_config_reader;


// Re-export legacy context types for backward compatibility
pub use self::siddhi_app_context::SiddhiAppContext;
pub use self::siddhi_context::SiddhiContext;
pub use self::siddhi_on_demand_query_context::SiddhiOnDemandQueryContext;
pub use self::siddhi_query_context::SiddhiQueryContext;
pub use self::statistics_configuration::StatisticsConfiguration;

// Re-export main configuration types for easy access
pub use types::*;
pub use manager::ConfigManager;
pub use error::{ConfigError, ConfigResult, ValidationResult, ValidationError};
pub use processor_config_reader::{ProcessorConfigReader, ConfigValue};

// Main configuration loading functions for simple usage
use std::path::Path;

/// Load configuration from the default locations with automatic environment detection
pub async fn load_config() -> ConfigResult<SiddhiConfig> {
    let manager = ConfigManager::new();
    manager.load_unified_config().await
}

/// Load configuration from a specific YAML file
pub async fn load_config_from_file<P: AsRef<Path>>(path: P) -> ConfigResult<SiddhiConfig> {
    let manager = ConfigManager::new();
    manager.load_from_file(path).await
}

/// Load configuration with custom manager settings
pub async fn load_config_with_manager(manager: ConfigManager) -> ConfigResult<SiddhiConfig> {
    manager.load_unified_config().await
}

// Re-exporting placeholders from siddhi_context.rs for now if they are widely used,
// though ideally they'd be replaced by actual types from their own modules.
// Example:
// pub use self::siddhi_context::{
//     PersistenceStorePlaceholder,
//     DataSourcePlaceholder,
//     // etc.
// };
// For SiddhiAppContext placeholders:
// pub use self::siddhi_app_context::{
//     StatisticsManagerPlaceholder,
//     TimestampGeneratorPlaceholder,
//     // etc.
// };
