pub mod join_processor;
pub use join_processor::{JoinProcessor, JoinProcessorSide, JoinSide};

#[derive(Debug)]
pub struct JoinStreamRuntime {
    pub join_processor: std::sync::Arc<std::sync::Mutex<JoinProcessor>>,
}
