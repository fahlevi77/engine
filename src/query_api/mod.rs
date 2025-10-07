pub mod aggregation;
pub mod annotation;
pub mod constants;
pub mod definition;
pub mod error;
pub mod eventflux_app;
pub mod eventflux_element;
pub mod execution;
pub mod expression; // Added aggregation module

// Re-export key types if desired, e.g.
pub use self::aggregation::Within;
pub use self::constants::*;
pub use self::definition::StreamDefinition;
pub use self::eventflux_app::EventFluxApp;
pub use self::eventflux_element::EventFluxElement; // Re-exporting Within
