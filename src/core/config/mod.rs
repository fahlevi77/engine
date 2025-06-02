pub mod statistics_configuration;
pub mod siddhi_context;
pub mod siddhi_app_context;
pub mod siddhi_query_context;
pub mod siddhi_on_demand_query_context;

pub use self::statistics_configuration::StatisticsConfiguration;
pub use self::siddhi_context::SiddhiContext;
pub use self::siddhi_app_context::SiddhiAppContext;
pub use self::siddhi_query_context::SiddhiQueryContext;
pub use self::siddhi_on_demand_query_context::SiddhiOnDemandQueryContext;

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
