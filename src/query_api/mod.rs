pub mod aggregation;
pub mod annotation;
pub mod constants;
pub mod definition;
pub mod error;
pub mod execution;
pub mod expression;
pub mod siddhi_app;
pub mod siddhi_element; // Added aggregation module

// Re-export key types if desired, e.g.
pub use self::aggregation::Within;
pub use self::constants::*;
pub use self::definition::StreamDefinition;
pub use self::siddhi_app::SiddhiApp;
pub use self::siddhi_element::SiddhiElement; // Re-exporting Within
