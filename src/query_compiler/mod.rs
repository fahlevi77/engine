pub mod siddhi_compiler;
// pub mod error; // If specific compiler errors are defined later

// Re-export all public functions from siddhi_compiler.rs module
// This makes them accessible as crate::query_compiler::parse, etc.
pub use self::siddhi_compiler::*;

// Or, if we prefer a struct-based approach for the compiler in Rust eventually:
// pub use self::siddhi_compiler::SiddhiCompiler;
// And then functions would be SiddhiCompiler::parse(), etc.
// For now, free functions are used as per the direct translation of static Java methods.
