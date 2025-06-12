pub mod sequence_processor;
pub mod logical_processor;

pub use sequence_processor::{SequenceProcessor, SequenceProcessorSide, SequenceType, SequenceSide};
pub use logical_processor::{LogicalProcessor, LogicalProcessorSide, LogicalType};
