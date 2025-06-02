pub mod siddhi_element;
pub mod siddhi_app;
pub mod definition;
pub mod expression;
pub mod execution;
pub mod annotation;
pub mod constants;
pub mod error;
pub mod aggregation; // Added aggregation module

// Re-export key types if desired, e.g.
pub use self::siddhi_app::SiddhiApp;
pub use self::definition::StreamDefinition;
pub use self::siddhi_element::SiddhiElement;
pub use self::constants::*;
pub use self::aggregation::Within; // Re-exporting Within
