// siddhi_rust/src/core/util/mod.rs

pub mod attribute_converter;
pub mod event_serde;
pub mod executor_service;
pub mod id_generator;
pub mod metrics;
pub mod parser; // Added parser module
pub mod scheduled_executor_service;
pub mod scheduler; // new scheduler module
pub mod serialization;
pub mod siddhi_constants; // Added siddhi_constants module
pub mod state_holder;
pub mod thread_barrier;
// Potentially other existing util submodules:
// pub mod cache;
// pub mod collection;
// pub mod config; // This might conflict with core::config
// pub mod error;  // This might conflict with core::exception or query_api::error
// pub mod event;  // This might conflict with core::event
// pub mod extension;
pub mod lock;
// pub mod persistence;
pub mod snapshot;
pub mod statistics; // This might conflict with core::config::StatisticsConfiguration
                    // pub mod timestamp;
                    // pub mod transport;

pub use self::attribute_converter::{get_property_value, get_property_value_from_str};
pub use self::event_serde::{event_from_bytes, event_to_bytes};
pub use self::executor_service::{ExecutorService, ExecutorServiceRegistry};
pub use self::id_generator::IdGenerator;
pub use self::lock::{LockSynchronizer, LockWrapper};
pub use self::metrics::*;
pub use self::parser::{parse_expression, ExpressionParserContext}; // Re-export key items from parser
pub use self::scheduled_executor_service::ScheduledExecutorService;
pub use self::scheduler::{Schedulable, Scheduler};
pub use self::serialization::{from_bytes, to_bytes};
pub use self::siddhi_constants::SiddhiConstants; // Re-export SiddhiConstants
pub use self::snapshot::{IncrementalSnapshot, PersistenceReference};
pub use self::state_holder::StateHolder;
pub use self::statistics::{DefaultStatisticsManager, StatisticsManager};
pub use self::thread_barrier::ThreadBarrier;
