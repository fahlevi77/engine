// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod join_processor;
pub mod table_join_processor;
pub use join_processor::{JoinProcessor, JoinProcessorSide, JoinSide};
pub use table_join_processor::TableJoinProcessor;

#[derive(Debug)]
pub struct JoinStreamRuntime {
    pub join_processor: std::sync::Arc<std::sync::Mutex<JoinProcessor>>,
}
