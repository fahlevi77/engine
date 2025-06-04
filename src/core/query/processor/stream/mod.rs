// siddhi_rust/src/core/query/processor/stream/mod.rs
pub mod filter;
// pub mod single; // If SingleStreamProcessor becomes a struct/module later
// pub mod window; // For window processors
// pub mod function; // For stream function processors

pub use self::filter::FilterProcessor;
// Other StreamProcessor types would be re-exported here.

// AbstractStreamProcessor.java and StreamProcessor.java (interface-like abstract class) from Java
// are conceptually covered by the Processor trait in core/query/processor.rs and
// the CommonProcessorMeta struct for shared fields. Specific stream processor types
// like FilterProcessor, WindowProcessor, etc., will implement the Processor trait.
