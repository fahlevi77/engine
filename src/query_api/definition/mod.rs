pub mod abstract_definition;
pub mod aggregation_definition;
pub mod attribute;
pub mod function_definition;
pub mod stream_definition;
pub mod table_definition;
pub mod trigger_definition;
pub mod window_definition;

pub use self::abstract_definition::AbstractDefinition;
pub use self::aggregation_definition::AggregationDefinition; // Keep this
pub use crate::query_api::aggregation::TimePeriod as AggregationTimePeriod; // Import directly
pub use self::attribute::{Attribute, Type as AttributeType};
pub use self::function_definition::FunctionDefinition;
pub use self::stream_definition::StreamDefinition;
pub use self::table_definition::TableDefinition;
pub use self::trigger_definition::TriggerDefinition;
pub use self::window_definition::WindowDefinition;
